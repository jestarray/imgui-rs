use crate::flags::Bindgen;
use anyhow::{anyhow, Context, Result};
use bindgen::Builder;
use std::fs;
use std::path::{Path, PathBuf};

impl Bindgen {
    pub fn run(self) -> Result<()> {
        let root = crate::project_root();

        let output = self
            .output_path
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("imgui-sys/src"));

        let wasm_name = self
            .wasm_import_name
            .or_else(|| std::env::var("IMGUI_RS_WASM_IMPORT_NAME").ok())
            .unwrap_or_else(|| "imgui-sys-v0".to_string());

        for variant in ["master", "docking"] {
            for flag in [None, Some("freetype")] {
                let additional = match flag {
                    None => "".to_string(),
                    Some(x) => format!("-{}", x),
                };
                let cimgui_output = root.join(format!(
                    "imgui-sys/third-party/imgui-{}{}",
                    variant, additional
                ));

                let (types, enum_names) = get_types(&cimgui_output.join("structs_and_enums.json"))?;
                let funcs = get_definitions(&cimgui_output.join("definitions.json"))?;
                let header = cimgui_output.join("cimgui.h");

                let output_name = match (variant, flag) {
                    ("master", None) => "bindings.rs".to_string(),
                    ("master", Some(f)) => format!("{}_bindings.rs", f),
                    (var, None) => format!("{}_bindings.rs", var),
                    (var, Some(f)) => format!("{}_{}_bindings.rs", var, f),
                };

                generate_binding_file(
                    &header,
                    &output.join(&output_name),
                    &types,
                    &enum_names,
                    &funcs,
                    None,
                )?;
                generate_binding_file(
                    &header,
                    &output.join(format!("wasm_{}", &output_name)),
                    &types,
                    &enum_names,
                    &funcs,
                    Some(&wasm_name),
                )?;
            }
        }

        Ok(())
    }
}

fn get_types(structs_and_enums: &Path) -> Result<(Vec<String>, Vec<String>)> {
    let types_txt = std::fs::read_to_string(structs_and_enums)?;
    let types_val = types_txt
        .parse::<smoljson::ValOwn>()
        .map_err(|e| anyhow!("Failed to parse {}: {:?}", structs_and_enums.display(), e))?;
    let enum_names: Vec<String> = types_val["enums"]
        .as_object()
        .ok_or_else(|| anyhow!("No `enums` in bindings file"))?
        .keys()
        .map(|k| k.to_string())
        .collect();
    let mut types: Vec<String> = enum_names.iter().map(|name| format!("^{name}")).collect();
    types.extend(
        types_val["structs"]
            .as_object()
            .ok_or_else(|| anyhow!("No `structs` in bindings file"))?
            .keys()
            .map(|k| format!("^{}", k)),
    );
    Ok((types, enum_names))
}

fn get_definitions(definitions: &Path) -> Result<Vec<String>> {
    fn bad_arg_type(s: &str) -> bool {
        s == "va_list" || s.starts_with("__")
    }
    let defs_txt = std::fs::read_to_string(definitions)?;
    let defs_val = defs_txt
        .parse::<smoljson::ValOwn>()
        .map_err(|e| anyhow!("Failed to parse {}: {:?}", definitions.display(), e))?;
    let definitions = defs_val
        .into_object()
        .ok_or_else(|| anyhow!("bad json data in defs file"))?;
    let mut keep_defs = vec![];
    for (name, def) in definitions {
        let defs = def
            .into_array()
            .ok_or_else(|| anyhow!("def {} not an array", &name))?;
        keep_defs.reserve(defs.len());
        for func in defs {
            let args = func["argsT"].as_array().unwrap();
            if !args
                .iter()
                .any(|a| a["type"].as_str().map_or(false, bad_arg_type))
            {
                let name = func["ov_cimguiname"]
                    .as_str()
                    .ok_or_else(|| anyhow!("ov_cimguiname wasnt string..."))?;
                keep_defs.push(format!("^{}", name));
            }
        }
    }
    Ok(keep_defs)
}

fn generate_binding_file(
    header: &Path,
    output: &Path,
    types: &[String],
    enum_names: &[String],
    funcs: &[String],
    wasm_import_mod: Option<&str>,
) -> Result<()> {
    let mut builder = Builder::default()
        .header(header.to_string_lossy())
        .size_t_is_usize(true)
        .prepend_enum_name(false)
        .generate_comments(false)
        // Layout tests aren't portable (they hardcode type sizes), and for
        // our case they just serve to sanity check rustc's implementation of
        // `#[repr(C)]`. If we bind directly to C++ ever, we should reconsider this.
        .layout_tests(false)
        .derive_default(true)
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .impl_debug(true)
        .use_core()
        .blocklist_type("__darwin_size_t")
        .raw_line("#![allow(nonstandard_style, clippy::all)]")
        .clang_arg("-DCIMGUI_DEFINE_ENUMS_AND_STRUCTS=1")
        .clang_arg("-DIMGUI_USE_WCHAR32=1");

    if let Some(name) = wasm_import_mod {
        builder = builder.wasm_import_module_name(name);
    }
    for t in types {
        builder = builder.allowlist_type(t);
    }
    for f in funcs {
        builder = builder.allowlist_function(f);
    }

    eprintln!("Executing bindgen [output = {}]", output.display());
    let bindings = builder.generate().context("Failed to execute bindgen")?;
    bindings
        .write_to_file(output)
        .context("Failed to write bindings")?;
    patch_bindings(output, enum_names)?;
    eprintln!("Success [output = {}]", output.display());

    Ok(())
}

fn patch_bindings(output: &Path, enum_names: &[String]) -> Result<()> {
    let text = fs::read_to_string(output)
        .with_context(|| format!("Failed to read bindings from {}", output.display()))?;

    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let mut changed = false;

    // Newer Clang versions infer unsigned enum aliases here. Keep the signed
    // aliases expected by imgui-rs and the existing generated bindings.
    for name in enum_names {
        let unsigned = format!("pub type {name} = ::core::ffi::c_uint;");
        if let Some(line) = lines.iter_mut().find(|line| **line == unsigned) {
            *line = format!("pub type {name} = ::core::ffi::c_int;");
            changed = true;
        }
    }

    let structs = [
        "ImFontLoader",
        "ImGuiSelectionBasicStorage",
        "ImGuiSelectionExternalStorage",
        "ImDrawCmd",
        "ImGuiPlatformIO",
    ];
    for name in structs {
        changed |= patch_struct(&mut lines, name)?;
    }

    if changed {
        fs::write(output, lines.join("\n") + "\n")
            .with_context(|| format!("Failed to write patched bindings to {}", output.display()))?;
    }
    Ok(())
}

fn patch_struct(lines: &mut Vec<String>, name: &str) -> Result<bool> {
    let struct_index = match lines.iter().position(|line| line.contains(&format!("pub struct {name}"))) {
        Some(index) => index,
        None => return Ok(false),
    };
    if lines.iter().any(|line| line.starts_with(&format!("impl PartialEq for {name}"))) {
        return Ok(false);
    }

    let mut derive_index = None;
    for i in (struct_index.saturating_sub(6)..struct_index).rev() {
        if lines[i].contains("#[derive(") {
            derive_index = Some(i);
            break;
        }
    }
    let derive_index = match derive_index {
        Some(index) => index,
        None => return Ok(false),
    };
    let derive_traits = parse_derive_traits(&lines[derive_index]);
    let needs_partialeq = derive_traits.iter().any(|item| item == "PartialEq");
    let needs_eq = derive_traits.iter().any(|item| item == "Eq");
    let needs_hash = derive_traits.iter().any(|item| item == "Hash");
    if !needs_partialeq && !needs_hash && !needs_eq {
        return Ok(false);
    }

    let fields = parse_struct_fields(lines, struct_index);
    if fields.is_empty() {
        return Ok(false);
    }

    lines[derive_index] = filter_derives(&lines[derive_index], &["Hash", "PartialEq", "Eq"]);

    let impl_block = build_impl_block(name, &fields, needs_partialeq, needs_eq, needs_hash);
    if impl_block.is_empty() {
        return Ok(false);
    }

    let insert_at = find_impl_end(lines, &format!("impl Default for {name}"))
        .or_else(|| find_struct_end(lines, struct_index))
        .ok_or_else(|| anyhow!("Failed to find insertion point for {name}"))?;

    lines.insert(insert_at + 1, impl_block);
    Ok(true)
}

fn find_impl_end(lines: &[String], needle: &str) -> Option<usize> {
    let impl_index = lines.iter().position(|line| line.starts_with(needle))?;
    let mut brace_depth = 0usize;
    for (i, line) in lines.iter().enumerate().skip(impl_index) {
        brace_depth += line.matches('{').count();
        brace_depth = brace_depth.saturating_sub(line.matches('}').count());
        if brace_depth == 0 && i > impl_index {
            return Some(i);
        }
    }
    None
}

fn find_struct_end(lines: &[String], struct_index: usize) -> Option<usize> {
    for (i, line) in lines.iter().enumerate().skip(struct_index) {
        if line.trim_start().starts_with('}') {
            return Some(i);
        }
    }
    None
}

#[derive(Debug)]
struct Field {
    name: String,
    ty: String,
}

fn parse_struct_fields(lines: &[String], struct_index: usize) -> Vec<Field> {
    let mut fields = Vec::new();
    let mut current = None;
    for line in lines.iter().skip(struct_index + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            break;
        }
        if trimmed.starts_with("pub ") {
            current = Some(trimmed.to_string());
        } else if let Some(buf) = current.as_mut() {
            if !trimmed.is_empty() {
                buf.push(' ');
                buf.push_str(trimmed);
            }
        }
        if let Some(buf) = current.as_ref() {
            if trimmed.ends_with(',') {
                if let Some(field) = parse_field(buf) {
                    fields.push(field);
                }
                current = None;
            }
        }
    }
    fields
}

fn parse_field(text: &str) -> Option<Field> {
    let text = text.trim().trim_end_matches(',');
    let text = text.strip_prefix("pub ")?;
    let mut parts = text.splitn(2, ':');
    let name = parts.next()?.trim().to_string();
    let ty = parts.next()?.trim().to_string();
    Some(Field { name, ty })
}

fn build_impl_block(
    name: &str,
    fields: &[Field],
    needs_partialeq: bool,
    needs_eq: bool,
    needs_hash: bool,
) -> String {
    let mut eq_checks = Vec::new();
    let mut hash_lines = Vec::new();
    let mut uses_fn_opt = false;
    let mut uses_fn = false;

    for field in fields {
        let is_fn_ptr = is_fn_ptr_type(&field.ty);
        let is_fn_ptr_option = is_fn_ptr_option_type(&field.ty);
        if is_fn_ptr {
            if is_fn_ptr_option {
                uses_fn_opt = true;
                eq_checks.push(format!("fn_opt_eq!(self.{0}, other.{0})", field.name));
                hash_lines.push(format!("fn_opt_hash!(self.{0}, state);", field.name));
            } else {
                uses_fn = true;
                eq_checks.push(format!("fn_eq!(self.{0}, other.{0})", field.name));
                hash_lines.push(format!("fn_hash!(self.{0}, state);", field.name));
            }
        } else {
            eq_checks.push(format!("self.{0} == other.{0}", field.name));
            hash_lines.push(format!("::core::hash::Hash::hash(&self.{0}, state);", field.name));
        }
    }

    let mut blocks = Vec::new();

    if needs_partialeq {
        let mut partial_eq = String::new();
        partial_eq.push_str(&format!("impl PartialEq for {name} {{\n"));
        partial_eq.push_str("    fn eq(&self, other: &Self) -> bool {\n");
        if uses_fn_opt {
            partial_eq.push_str(
                "        macro_rules! fn_opt_eq {\n\
            ($left:expr, $right:expr) => {\n\
                match ($left, $right) {\n\
                    (Some(a), Some(b)) => ::core::ptr::fn_addr_eq(a, b),\n\
                    (None, None) => true,\n\
                    _ => false,\n\
                }\n\
            };\n\
        }\n\n",
            );
        }
        if uses_fn {
            partial_eq.push_str(
                "        macro_rules! fn_eq {\n\
            ($left:expr, $right:expr) => {\n\
                ::core::ptr::fn_addr_eq($left, $right)\n\
            };\n\
        }\n\n",
            );
        }
        if eq_checks.is_empty() {
            partial_eq.push_str("        true\n");
        } else {
            partial_eq.push_str(&format!("        {}\n", eq_checks.join("\n            && ")));
        }
        partial_eq.push_str("    }\n}\n");
        blocks.push(partial_eq);
    }

    if needs_eq {
        blocks.push(format!("impl Eq for {name} {{}}\n"));
    }

    if needs_hash {
        let mut hash = String::new();
        hash.push_str(&format!("impl ::core::hash::Hash for {name} {{\n"));
        hash.push_str("    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {\n");
        if uses_fn_opt {
            hash.push_str(
                "        macro_rules! fn_opt_hash {\n\
            ($value:expr, $state:expr) => {{\n\
                let addr = match $value {\n\
                    Some(f) => f as *const () as usize,\n\
                    None => 0,\n\
                };\n\
                ::core::hash::Hash::hash(&addr, $state);\n\
            }};\n\
        }\n\n",
            );
        }
        if uses_fn {
            hash.push_str(
                "        macro_rules! fn_hash {\n\
            ($value:expr, $state:expr) => {{\n\
                let addr = $value as *const () as usize;\n\
                ::core::hash::Hash::hash(&addr, $state);\n\
            }};\n\
        }\n\n",
            );
        }
        if hash_lines.is_empty() {
            hash.push_str("        let _ = state;\n");
        } else {
            for line in hash_lines {
                hash.push_str("        ");
                hash.push_str(&line);
                hash.push('\n');
            }
        }
        hash.push_str("    }\n}\n");
        blocks.push(hash);
    }

    blocks.join("\n")
}

fn is_fn_ptr_type(ty: &str) -> bool {
    ty.contains("extern \"C\" fn") || ty == "ImDrawCallback"
}

fn is_fn_ptr_option_type(ty: &str) -> bool {
    ty.contains("Option<") || ty == "ImDrawCallback"
}

fn parse_derive_traits(line: &str) -> Vec<String> {
    let start = line.find('(');
    let end = line.rfind(')');
    if let (Some(start), Some(end)) = (start, end) {
        line[start + 1..end]
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    } else {
        Vec::new()
    }
}

fn filter_derives(line: &str, remove: &[&str]) -> String {
    let start = line.find('(');
    let end = line.rfind(')');
    if let (Some(start), Some(end)) = (start, end) {
        let inside = &line[start + 1..end];
        let filtered: Vec<&str> = inside
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !remove.iter().any(|r| r == item))
            .collect();
        let prefix = &line[..start + 1];
        let suffix = &line[end..];
        format!("{}{}{}", prefix, filtered.join(", "), suffix)
    } else {
        line.to_string()
    }
}
