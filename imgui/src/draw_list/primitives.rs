//! Additional primitives and independently buffered paths.
use super::{DrawFlags, DrawListMut};
use crate::{math::MintVec2, sys, ImColor32};

fn count(value: u32) -> i32 {
    i32::try_from(value).expect("segment count exceeds i32")
}
fn point(value: impl Into<MintVec2>) -> sys::ImVec2 {
    let value: sys::ImVec2 = value.into().into();
    assert!(
        value.x.is_finite() && value.y.is_finite(),
        "coordinates must be finite"
    );
    value
}
fn nonnegative(value: f32) {
    assert!(
        value.is_finite() && value >= 0.0,
        "value must be finite and nonnegative"
    );
}

impl<'ui> DrawListMut<'ui> {
    /// Build a horizontal line with inside stroke positioning by default.
    #[doc(alias = "AddLineH")]
    pub fn add_line_h(
        &'ui self,
        min_x: f32,
        max_x: f32,
        y: f32,
        color: impl Into<ImColor32>,
    ) -> AxisLine<'ui> {
        assert!(min_x.is_finite());
        assert!(max_x.is_finite());
        assert!(y.is_finite());
        AxisLine {
            draw: self,
            min_x,
            max_x,
            y,
            color: color.into(),
            thickness: 1.0,
            flags: DrawFlags::empty(),
            vertical: false,
        }
    }
    /// Build a quadrilateral. Call `.build()` to draw it.
    #[doc(alias = "AddQuad")]
    pub fn add_quad(
        &'ui self,
        p1: impl Into<MintVec2>,
        p2: impl Into<MintVec2>,
        p3: impl Into<MintVec2>,
        p4: impl Into<MintVec2>,
        color: impl Into<ImColor32>,
    ) -> Quad<'ui> {
        let p1 = point(p1);
        let p2 = point(p2);
        let p3 = point(p3);
        let p4 = point(p4);
        Quad {
            draw: self,
            p1,
            p2,
            p3,
            p4,
            color: color.into(),
            thickness: 1.0,
            flags: DrawFlags::empty(),
            filled: false,
        }
    }
    /// Build a regular polygon with at least three sides.
    #[doc(alias = "AddNgon")]
    pub fn add_ngon(
        &'ui self,
        center: impl Into<MintVec2>,
        radius: f32,
        num_segments: u32,
        color: impl Into<ImColor32>,
    ) -> Ngon<'ui> {
        let center = point(center);
        assert!(radius.is_finite());
        assert!(num_segments >= 3);
        count(num_segments);
        nonnegative(radius);
        Ngon {
            draw: self,
            center,
            radius,
            num_segments,
            color: color.into(),
            thickness: 1.0,
            flags: DrawFlags::empty(),
            filled: false,
        }
    }
    /// Build an ellipse. Call `.build()` to draw it.
    #[doc(alias = "AddEllipse")]
    pub fn add_ellipse(
        &'ui self,
        center: impl Into<MintVec2>,
        radius: impl Into<MintVec2>,
        color: impl Into<ImColor32>,
    ) -> Ellipse<'ui> {
        let center = point(center);
        let radius = point(radius);
        nonnegative(radius.x);
        nonnegative(radius.y);
        Ellipse {
            draw: self,
            center,
            radius,
            color: color.into(),
            thickness: 1.0,
            flags: DrawFlags::empty(),
            num_segments: 0,
            rotation: 0.0,
            filled: false,
        }
    }
    /// Build a quadratic Bézier curve. Call `.build()` to draw it.
    #[doc(alias = "AddBezierQuadratic")]
    pub fn add_bezier_quadratic(
        &'ui self,
        p1: impl Into<MintVec2>,
        p2: impl Into<MintVec2>,
        p3: impl Into<MintVec2>,
        color: impl Into<ImColor32>,
    ) -> BezierQuadratic<'ui> {
        let p1 = point(p1);
        let p2 = point(p2);
        let p3 = point(p3);
        BezierQuadratic {
            draw: self,
            p1,
            p2,
            p3,
            color: color.into(),
            thickness: 1.0,
            flags: DrawFlags::empty(),
            num_segments: 0,
        }
    }
    /// Build an axis-aligned vertical line with inside stroke positioning by default.
    #[doc(alias = "AddLineV")]
    pub fn add_line_v(
        &'ui self,
        x: f32,
        min_y: f32,
        max_y: f32,
        color: impl Into<ImColor32>,
    ) -> AxisLine<'ui> {
        let mut line = self.add_line_h(min_y, max_y, x, color);
        line.vertical = true;
        line
    }
    /// Fill a simple concave polygon with clockwise winding. Holes are unsupported.
    #[doc(alias = "AddConcavePolyFilled")]
    pub fn add_concave_poly_filled(
        &self,
        points: impl IntoIterator<Item = impl Into<MintVec2>>,
        color: impl Into<ImColor32>,
    ) {
        let points: Vec<_> = points.into_iter().map(point).collect();
        let len = i32::try_from(points.len()).expect("too many points");
        if len >= 3 {
            unsafe {
                sys::ImDrawList_AddConcavePolyFilled(
                    self.draw_list,
                    points.as_ptr(),
                    len,
                    color.into().into(),
                )
            };
        }
    }
    /// Start an independent path. No shared draw-list state changes until it is finished.
    pub fn path(&'ui self) -> Path<'ui> {
        Path {
            draw: self,
            commands: Vec::new(),
        }
    }
}

/// A horizontal or vertical line waiting to be drawn.
#[must_use = "call build to draw"]
pub struct AxisLine<'ui> {
    draw: &'ui DrawListMut<'ui>,
    color: ImColor32,
    thickness: f32,
    flags: DrawFlags,
    min_x: f32,
    max_x: f32,
    y: f32,
    vertical: bool,
}
impl AxisLine<'_> {
    /// Set stroke thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        nonnegative(value);
        self.thickness = value;
        self
    }
    /// Set primitive flags; choose only one stroke position.
    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Append this primitive to the draw list.
    pub fn build(self) {
        unsafe {
            if self.vertical {
                sys::ImDrawList_AddLineV(
                    self.draw.draw_list,
                    self.y,
                    self.min_x,
                    self.max_x,
                    self.color.into(),
                    self.thickness,
                    self.flags.bits(),
                );
            } else {
                sys::ImDrawList_AddLineH(
                    self.draw.draw_list,
                    self.min_x,
                    self.max_x,
                    self.y,
                    self.color.into(),
                    self.thickness,
                    self.flags.bits(),
                );
            }
        }
    }
}

/// A quadrilateral waiting to be drawn.
#[must_use = "call build to draw"]
pub struct Quad<'ui> {
    draw: &'ui DrawListMut<'ui>,
    color: ImColor32,
    thickness: f32,
    flags: DrawFlags,
    p1: sys::ImVec2,
    p2: sys::ImVec2,
    p3: sys::ImVec2,
    p4: sys::ImVec2,
    filled: bool,
}
impl Quad<'_> {
    /// Set stroke thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        nonnegative(value);
        self.thickness = value;
        self
    }
    /// Set primitive flags; choose only one stroke position.
    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Fill the shape instead of stroking it. Stroke-only flags have no effect on fills.
    pub fn filled(mut self, value: bool) -> Self {
        self.filled = value;
        self
    }
    /// Append this primitive to the draw list.
    pub fn build(self) {
        unsafe {
            if self.filled {
                sys::ImDrawList_AddQuadFilled(
                    self.draw.draw_list,
                    self.p1,
                    self.p2,
                    self.p3,
                    self.p4,
                    self.color.into(),
                );
            } else {
                sys::ImDrawList_AddQuad(
                    self.draw.draw_list,
                    self.p1,
                    self.p2,
                    self.p3,
                    self.p4,
                    self.color.into(),
                    self.thickness,
                    self.flags.bits(),
                );
            }
        }
    }
}

/// A regular polygon waiting to be drawn.
#[must_use = "call build to draw"]
pub struct Ngon<'ui> {
    draw: &'ui DrawListMut<'ui>,
    color: ImColor32,
    thickness: f32,
    flags: DrawFlags,
    center: sys::ImVec2,
    radius: f32,
    num_segments: u32,
    filled: bool,
}
impl Ngon<'_> {
    /// Set stroke thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        nonnegative(value);
        self.thickness = value;
        self
    }
    /// Set primitive flags; choose only one stroke position.
    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Fill the shape instead of stroking it. Stroke-only flags have no effect on fills.
    pub fn filled(mut self, value: bool) -> Self {
        self.filled = value;
        self
    }
    /// Append this primitive to the draw list.
    pub fn build(self) {
        unsafe {
            if self.filled {
                sys::ImDrawList_AddNgonFilled(
                    self.draw.draw_list,
                    self.center,
                    self.radius,
                    self.color.into(),
                    count(self.num_segments),
                );
            } else {
                sys::ImDrawList_AddNgon(
                    self.draw.draw_list,
                    self.center,
                    self.radius,
                    self.color.into(),
                    count(self.num_segments),
                    self.thickness,
                    self.flags.bits(),
                );
            }
        }
    }
}

/// An ellipse waiting to be drawn.
#[must_use = "call build to draw"]
pub struct Ellipse<'ui> {
    draw: &'ui DrawListMut<'ui>,
    color: ImColor32,
    thickness: f32,
    flags: DrawFlags,
    center: sys::ImVec2,
    radius: sys::ImVec2,
    num_segments: u32,
    rotation: f32,
    filled: bool,
}
impl Ellipse<'_> {
    /// Set stroke thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        nonnegative(value);
        self.thickness = value;
        self
    }
    /// Set primitive flags; choose only one stroke position.
    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Fill the shape instead of stroking it. Stroke-only flags have no effect on fills.
    pub fn filled(mut self, value: bool) -> Self {
        self.filled = value;
        self
    }
    /// Set tessellation segments; zero selects automatic tessellation.
    pub fn num_segments(mut self, value: u32) -> Self {
        count(value);
        self.num_segments = value;
        self
    }
    /// Rotate the ellipse in radians.
    pub fn rotation(mut self, value: f32) -> Self {
        assert!(value.is_finite());
        self.rotation = value;
        self
    }
    /// Append this primitive to the draw list.
    pub fn build(self) {
        unsafe {
            if self.filled {
                sys::ImDrawList_AddEllipseFilled(
                    self.draw.draw_list,
                    self.center,
                    self.radius,
                    self.color.into(),
                    self.rotation,
                    count(self.num_segments),
                );
            } else {
                sys::ImDrawList_AddEllipse(
                    self.draw.draw_list,
                    self.center,
                    self.radius,
                    self.color.into(),
                    self.rotation,
                    count(self.num_segments),
                    self.thickness,
                    self.flags.bits(),
                );
            }
        }
    }
}

/// A quadratic Bézier curve waiting to be drawn.
#[must_use = "call build to draw"]
pub struct BezierQuadratic<'ui> {
    draw: &'ui DrawListMut<'ui>,
    color: ImColor32,
    thickness: f32,
    flags: DrawFlags,
    p1: sys::ImVec2,
    p2: sys::ImVec2,
    p3: sys::ImVec2,
    num_segments: u32,
}
impl BezierQuadratic<'_> {
    /// Set stroke thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        nonnegative(value);
        self.thickness = value;
        self
    }
    /// Set primitive flags; choose only one stroke position.
    pub fn flags(mut self, flags: DrawFlags) -> Self {
        self.flags = flags;
        self
    }
    /// Set tessellation segments; zero selects automatic tessellation.
    pub fn num_segments(mut self, value: u32) -> Self {
        count(value);
        self.num_segments = value;
        self
    }
    /// Append this primitive to the draw list.
    pub fn build(self) {
        unsafe {
            sys::ImDrawList_AddBezierQuadratic(
                self.draw.draw_list,
                self.p1,
                self.p2,
                self.p3,
                self.color.into(),
                self.thickness,
                count(self.num_segments),
                self.flags.bits(),
            );
        }
    }
}

enum Command {
    Line(sys::ImVec2, bool),
    Arc(sys::ImVec2, f32, f32, f32, i32),
    Ellipse(sys::ImVec2, sys::ImVec2, f32, f32, f32, i32),
    Cubic(sys::ImVec2, sys::ImVec2, sys::ImVec2, i32),
    Quadratic(sys::ImVec2, sys::ImVec2, i32),
    Rect(sys::ImVec2, sys::ImVec2, f32, DrawFlags),
    FastArc(sys::ImVec2, f32, i32, i32),
}
/// An independently buffered path. Dropping it without finishing draws nothing.
#[must_use = "finish the path with stroke or fill"]
pub struct Path<'ui> {
    draw: &'ui DrawListMut<'ui>,
    commands: Vec<Command>,
}
struct ClearPath(*mut sys::ImDrawList);
impl Drop for ClearPath {
    fn drop(&mut self) {
        unsafe { sys::ImDrawList_PathClear(self.0) };
    }
}
impl Path<'_> {
    /// Append a point.
    pub fn line_to(mut self, p: impl Into<MintVec2>) -> Self {
        self.commands.push(Command::Line(point(p), false));
        self
    }
    /// Append a point unless identical to the previous point.
    pub fn line_to_merge_duplicate(mut self, p: impl Into<MintVec2>) -> Self {
        self.commands.push(Command::Line(point(p), true));
        self
    }
    /// Append an arc with angles in radians. Zero segments enables automatic tessellation.
    pub fn arc_to(
        mut self,
        center: impl Into<MintVec2>,
        radius: f32,
        min: f32,
        max: f32,
        segments: u32,
    ) -> Self {
        nonnegative(radius);
        assert!(min.is_finite() && max.is_finite());
        self.commands.push(Command::Arc(
            point(center),
            radius,
            min,
            max,
            count(segments),
        ));
        self
    }
    /// Append an arc using the precomputed twelve-step circle.
    pub fn arc_to_fast(
        mut self,
        center: impl Into<MintVec2>,
        radius: f32,
        min: i32,
        max: i32,
    ) -> Self {
        nonnegative(radius);
        assert!((0..=12).contains(&min) && (0..=12).contains(&max));
        self.commands
            .push(Command::FastArc(point(center), radius, min, max));
        self
    }
    /// Append an elliptical arc. Rotation and angles are in radians.
    pub fn elliptical_arc_to(
        mut self,
        center: impl Into<MintVec2>,
        radius: impl Into<MintVec2>,
        rotation: f32,
        min: f32,
        max: f32,
        segments: u32,
    ) -> Self {
        let radius = point(radius);
        nonnegative(radius.x);
        nonnegative(radius.y);
        assert!(rotation.is_finite() && min.is_finite() && max.is_finite());
        self.commands.push(Command::Ellipse(
            point(center),
            radius,
            rotation,
            min,
            max,
            count(segments),
        ));
        self
    }
    /// Append a cubic Bezier segment from the previous point.
    pub fn bezier_cubic_to(
        mut self,
        p2: impl Into<MintVec2>,
        p3: impl Into<MintVec2>,
        p4: impl Into<MintVec2>,
        segments: u32,
    ) -> Self {
        assert!(!self.commands.is_empty(), "Bezier needs a starting point");
        self.commands.push(Command::Cubic(
            point(p2),
            point(p3),
            point(p4),
            count(segments),
        ));
        self
    }
    /// Append a quadratic Bezier segment from the previous point.
    pub fn bezier_quadratic_to(
        mut self,
        p2: impl Into<MintVec2>,
        p3: impl Into<MintVec2>,
        segments: u32,
    ) -> Self {
        assert!(!self.commands.is_empty(), "Bezier needs a starting point");
        self.commands
            .push(Command::Quadratic(point(p2), point(p3), count(segments)));
        self
    }
    /// Append a rectangle, with optional corner rounding.
    pub fn rect(
        mut self,
        min: impl Into<MintVec2>,
        max: impl Into<MintVec2>,
        rounding: f32,
        flags: DrawFlags,
    ) -> Self {
        nonnegative(rounding);
        self.commands
            .push(Command::Rect(point(min), point(max), rounding, flags));
        self
    }
    fn emit(&self) -> ClearPath {
        let clear = ClearPath(self.draw.draw_list);
        unsafe {
            sys::ImDrawList_PathClear(self.draw.draw_list);
            for command in &self.commands {
                let d = self.draw.draw_list;
                match *command {
                    Command::Line(p, false) => sys::ImDrawList_PathLineTo(d, p),
                    Command::Line(p, true) => sys::ImDrawList_PathLineToMergeDuplicate(d, p),
                    Command::Arc(c, r, a, b, n) => sys::ImDrawList_PathArcTo(d, c, r, a, b, n),
                    Command::FastArc(c, r, a, b) => sys::ImDrawList_PathArcToFast(d, c, r, a, b),
                    Command::Ellipse(c, r, rot, a, b, n) => {
                        sys::ImDrawList_PathEllipticalArcTo(d, c, r, rot, a, b, n)
                    }
                    Command::Cubic(a, b, c, n) => {
                        assert!((*d)._Path.Size > 0, "Bezier needs a starting point");
                        sys::ImDrawList_PathBezierCubicCurveTo(d, a, b, c, n)
                    }
                    Command::Quadratic(a, b, n) => {
                        assert!((*d)._Path.Size > 0, "Bezier needs a starting point");
                        sys::ImDrawList_PathBezierQuadraticCurveTo(d, a, b, n)
                    }
                    Command::Rect(a, b, r, f) => sys::ImDrawList_PathRect(d, a, b, r, f.bits()),
                }
            }
        }
        clear
    }
    /// Stroke this path; use `CLOSED` to join its last and first points.
    pub fn stroke(self, color: impl Into<ImColor32>, thickness: f32, flags: DrawFlags) {
        let color = color.into();
        nonnegative(thickness);
        let _clear = self.emit();
        unsafe {
            sys::ImDrawList_PathStroke(self.draw.draw_list, color.into(), thickness, flags.bits())
        };
    }
    /// Fill a convex path with clockwise winding.
    pub fn fill_convex(self, color: impl Into<ImColor32>, flags: DrawFlags) {
        let color = color.into();
        let _clear = self.emit();
        unsafe { sys::ImDrawList_PathFillConvex(self.draw.draw_list, color.into(), flags.bits()) };
    }
    /// Fill a simple concave path with clockwise winding and no holes.
    pub fn fill_concave(self, color: impl Into<ImColor32>) {
        let color = color.into();
        let _clear = self.emit();
        unsafe { sys::ImDrawList_PathFillConcave(self.draw.draw_list, color.into()) };
    }
}
