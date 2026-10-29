//! Art Page Border rendering implementations for WordCraft.

use std::sync::Arc;
use wordcraft_doc::freeform::{FreePath, Freeform};
use wordcraft_doc::para::ShapeKind;
use wordcraft_doc::props::{ArtStyle, Border, BorderStyle, Borders, Rgb};
use wordcraft_doc::section::SectionProps;
use wordcraft_geom::{Rect, Spin};

use crate::Placed;

/// Render art border rules for a section page.
pub fn art_border_rules(s: &SectionProps, b: &Borders, art: ArtStyle) -> Vec<Placed> {
    let mut items = Vec::new();

    // Determine side enablement. If a side is specified and explicitly None / 0 width, disabled.
    let top_side = b.top.as_ref();
    let bottom_side = b.bottom.as_ref();
    let left_side = b.left.as_ref();
    let right_side = b.right.as_ref();

    let top_on = top_side.map_or(true, |side| side.style != BorderStyle::None);
    let bottom_on = bottom_side.map_or(true, |side| side.style != BorderStyle::None);
    let left_on = left_side.map_or(true, |side| side.style != BorderStyle::None);
    let right_on = right_side.map_or(true, |side| side.style != BorderStyle::None);

    if !top_on && !bottom_on && !left_on && !right_on {
        return items;
    }

    // Determine offset/space from page edges.
    let sp = |e: Option<&Border>| e.map(|x| x.space).unwrap_or(24.0);
    let space_left = sp(left_side);
    let space_right = sp(right_side);
    let space_top = sp(top_side);
    let space_bottom = sp(bottom_side);

    // Page dimensions and border rectangle bounds.
    let x0 = space_left;
    let x1 = (s.page_w - space_right).max(x0 + 10.0);
    let y0 = space_top;
    let y1 = (s.page_h - space_bottom).max(y0 + 10.0);

    // Get border width and tint color if provided.
    let width_pt = top_side
        .or(bottom_side)
        .or(left_side)
        .or(right_side)
        .map(|side| side.width)
        .unwrap_or(24.0);
    let user_color = top_side
        .or(bottom_side)
        .or(left_side)
        .or(right_side)
        .and_then(|side| side.color);

    // Base tile size in points.
    let tile_size = (width_pt.clamp(4.0, 72.0) * 1.5).clamp(12.0, 72.0);

    // Corner bounds:
    let tl_rect = Rect::new(x0, y0, tile_size, tile_size);
    let tr_rect = Rect::new(x1 - tile_size, y0, tile_size, tile_size);
    let bl_rect = Rect::new(x0, y1 - tile_size, tile_size, tile_size);
    let br_rect = Rect::new(x1 - tile_size, y1 - tile_size, tile_size, tile_size);

    // 1. Draw Corners
    if top_on || left_on {
        render_corner_motif(&mut items, art, tl_rect, 0.0, user_color);
    }
    if top_on || right_on {
        render_corner_motif(&mut items, art, tr_rect, 90.0, user_color);
    }
    if bottom_on || right_on {
        render_corner_motif(&mut items, art, br_rect, 180.0, user_color);
    }
    if bottom_on || left_on {
        render_corner_motif(&mut items, art, bl_rect, 270.0, user_color);
    }

    // 2. Top Edge Tiles
    if top_on {
        let span = (tr_rect.x - (tl_rect.x + tile_size)).max(0.0);
        if span > 0.0 {
            let count = ((span / tile_size).round() as usize).max(1);
            let step = span / count as f32;
            for i in 0..count {
                let tx = tl_rect.x + tile_size + i as f32 * step;
                let rect = Rect::new(tx, y0, step, tile_size);
                render_edge_motif(&mut items, art, rect, 0.0, user_color);
            }
        }
    }

    // 3. Bottom Edge Tiles
    if bottom_on {
        let span = (br_rect.x - (bl_rect.x + tile_size)).max(0.0);
        if span > 0.0 {
            let count = ((span / tile_size).round() as usize).max(1);
            let step = span / count as f32;
            for i in 0..count {
                let tx = bl_rect.x + tile_size + i as f32 * step;
                let rect = Rect::new(tx, y1 - tile_size, step, tile_size);
                render_edge_motif(&mut items, art, rect, 180.0, user_color);
            }
        }
    }

    // 4. Left Edge Tiles
    if left_on {
        let span = (bl_rect.y - (tl_rect.y + tile_size)).max(0.0);
        if span > 0.0 {
            let count = ((span / tile_size).round() as usize).max(1);
            let step = span / count as f32;
            for i in 0..count {
                let ty = tl_rect.y + tile_size + i as f32 * step;
                let rect = Rect::new(x0, ty, tile_size, step);
                render_edge_motif(&mut items, art, rect, 270.0, user_color);
            }
        }
    }

    // 5. Right Edge Tiles
    if right_on {
        let span = (br_rect.y - (tr_rect.y + tile_size)).max(0.0);
        if span > 0.0 {
            let count = ((span / tile_size).round() as usize).max(1);
            let step = span / count as f32;
            for i in 0..count {
                let ty = tr_rect.y + tile_size + i as f32 * step;
                let rect = Rect::new(x1 - tile_size, ty, tile_size, step);
                render_edge_motif(&mut items, art, rect, 90.0, user_color);
            }
        }
    }

    items
}

/// Helper to render a corner motif into the items buffer.
fn render_corner_motif(items: &mut Vec<Placed>, art: ArtStyle, rect: Rect, deg: f32, user_color: Option<Rgb>) {
    render_art_motif(items, art, rect, deg, true, user_color);
}

/// Helper to render an edge tile motif into the items buffer.
fn render_edge_motif(items: &mut Vec<Placed>, art: ArtStyle, rect: Rect, deg: f32, user_color: Option<Rgb>) {
    render_art_motif(items, art, rect, deg, false, user_color);
}

/// Core dispatcher that constructs vector shapes for all 9 registered Art border styles.
fn render_art_motif(
    items: &mut Vec<Placed>,
    art: ArtStyle,
    rect: Rect,
    deg: f32,
    is_corner: bool,
    user_color: Option<Rgb>,
) {
    let spin = Spin::new(deg, false, false);

    match art {
        ArtStyle::WdArtFlowersBlockPrint => {
            // Block Flowers Print: Multi-petal geometric block floral tiles with leaf accents
            let primary = user_color.unwrap_or(Rgb(0xA0, 0x30, 0x20));
            let secondary = Rgb(0x40, 0x70, 0x30);
            let center = Rgb(0xF0, 0xC0, 0x40);

            // Base tile rectangle
            let petal_size = rect.w.min(rect.h) * 0.35;
            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;

            // Center block
            items.push(Placed::Shape {
                rect: Rect::new(cx - petal_size * 0.4, cy - petal_size * 0.4, petal_size * 0.8, petal_size * 0.8),
                kind: ShapeKind::Rectangle,
                fill: Some(center),
                stroke: Some(Rgb(0x30, 0x20, 0x10)),
                stroke_width: 1.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // 4 Block Petals
            for (dx, dy) in [(-0.8, -0.8), (0.8, -0.8), (-0.8, 0.8), (0.8, 0.8)] {
                let px = cx + dx * petal_size * 0.5 - petal_size * 0.35;
                let py = cy + dy * petal_size * 0.5 - petal_size * 0.35;
                items.push(Placed::Shape {
                    rect: Rect::new(px, py, petal_size * 0.7, petal_size * 0.7),
                    kind: ShapeKind::RoundedRectangle,
                    fill: Some(primary),
                    stroke: Some(Rgb(0x50, 0x10, 0x10)),
                    stroke_width: 0.75,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Corner / Edge Leaf block
            if is_corner {
                items.push(Placed::Shape {
                    rect: Rect::new(rect.x + 2.0, rect.y + 2.0, rect.w * 0.3, rect.h * 0.3),
                    kind: ShapeKind::Diamond,
                    fill: Some(secondary),
                    stroke: Some(Rgb(0x20, 0x40, 0x10)),
                    stroke_width: 0.75,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }
        }

        ArtStyle::WdArtFlowersDaisies => {
            // Daisies: White petals radiating around a bright yellow/gold center, green leaf stems
            let petal_color = Rgb(0xFF, 0xFF, 0xFF);
            let center_color = user_color.unwrap_or(Rgb(0xFF, 0xC0, 0x00));
            let stem_color = Rgb(0x50, 0x90, 0x40);

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let r = rect.w.min(rect.h) * 0.22;

            // Green leaf background line / vine
            items.push(Placed::Shape {
                rect: Rect::new(rect.x, cy - 1.0, rect.w, 2.0),
                kind: ShapeKind::Rectangle,
                fill: Some(stem_color),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // 8 Daisy Petals
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::PI / 4.0;
                let px = cx + angle.cos() * r * 1.2 - r * 0.35;
                let py = cy + angle.sin() * r * 1.2 - r * 0.35;
                items.push(Placed::Shape {
                    rect: Rect::new(px, py, r * 0.7, r * 0.7),
                    kind: ShapeKind::Ellipse,
                    fill: Some(petal_color),
                    stroke: Some(Rgb(0xB0, 0xB0, 0xB0)),
                    stroke_width: 0.5,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Yellow Center Disc
            items.push(Placed::Shape {
                rect: Rect::new(cx - r * 0.7, cy - r * 0.7, r * 1.4, r * 1.4),
                kind: ShapeKind::Ellipse,
                fill: Some(center_color),
                stroke: Some(Rgb(0xD0, 0x90, 0x00)),
                stroke_width: 0.75,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersModern1 => {
            // Modern Flowers 1: Elegant stylized tulips / poppies with sweeping stems
            let bloom_color = user_color.unwrap_or(Rgb(0xC0, 0x40, 0x70));
            let stem_color = Rgb(0x30, 0x60, 0x50);

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let sz = rect.w.min(rect.h) * 0.4;

            // Curved stem line
            items.push(Placed::Shape {
                rect: Rect::new(rect.x + 2.0, cy - 1.0, rect.w - 4.0, 2.0),
                kind: ShapeKind::Rectangle,
                fill: Some(stem_color),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Modern Tulip Cup Blossom
            let ff_pts = vec![
                [cx - sz * 0.5, cy + sz * 0.2],
                [cx - sz * 0.6, cy - sz * 0.4],
                [cx - sz * 0.2, cy - sz * 0.1],
                [cx, cy - sz * 0.5],
                [cx + sz * 0.2, cy - sz * 0.1],
                [cx + sz * 0.6, cy - sz * 0.4],
                [cx + sz * 0.5, cy + sz * 0.2],
                [cx, cy + sz * 0.6],
            ];

            let ff = Arc::new(Freeform {
                w: rect.w,
                h: rect.h,
                paths: vec![FreePath { pts: ff_pts, closed: true }],
                ink: None,
                order: None,
                alpha: 1.0,
            });

            items.push(Placed::Shape {
                rect,
                kind: ShapeKind::Freeform,
                fill: Some(bloom_color),
                stroke: Some(Rgb(0x70, 0x10, 0x30)),
                stroke_width: 1.0,
                freeform: Some(ff),
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersModern2 => {
            // Modern Flowers 2: Geometric starburst floral arrangement with layered circles
            let outer_color = user_color.unwrap_or(Rgb(0x20, 0x70, 0xB0));
            let inner_color = Rgb(0x70, 0xC0, 0xE0);
            let core_color = Rgb(0xFF, 0xFF, 0xFF);

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let r = rect.w.min(rect.h) * 0.4;

            // Outer starburst (8 points using diamond / rotated squares)
            for i in 0..4 {
                let angle = (i as f32) * std::f32::consts::PI / 4.0;
                let s_spin = Spin::new(deg + angle.to_degrees(), false, false);
                items.push(Placed::Shape {
                    rect: Rect::new(cx - r * 0.7, cy - r * 0.7, r * 1.4, r * 1.4),
                    kind: ShapeKind::Diamond,
                    fill: Some(outer_color),
                    stroke: Some(Rgb(0x10, 0x40, 0x70)),
                    stroke_width: 0.75,
                    freeform: None,
                    effects: Default::default(),
                    spin: s_spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Inner circle
            items.push(Placed::Shape {
                rect: Rect::new(cx - r * 0.45, cy - r * 0.45, r * 0.9, r * 0.9),
                kind: ShapeKind::Ellipse,
                fill: Some(inner_color),
                stroke: Some(core_color),
                stroke_width: 1.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersPansy => {
            // Pansy: Distinctive 5-petal pansy with characteristic dark velvet eye / center
            let top_petals = user_color.unwrap_or(Rgb(0x70, 0x30, 0xA0)); // Purple
            let bottom_petals = Rgb(0xFF, 0xC0, 0x00); // Yellow/Orange
            let eye_color = Rgb(0x30, 0x00, 0x40); // Dark velvet eye

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let sz = rect.w.min(rect.h) * 0.38;

            // Top 2 Purple Petals
            for dx in [-0.35, 0.35] {
                items.push(Placed::Shape {
                    rect: Rect::new(cx + dx * sz - sz * 0.45, cy - sz * 0.5, sz * 0.9, sz * 0.9),
                    kind: ShapeKind::Ellipse,
                    fill: Some(top_petals),
                    stroke: Some(Rgb(0x40, 0x10, 0x60)),
                    stroke_width: 0.5,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Side & Bottom 3 Yellow/Gold Petals
            for (dx, dy) in [(-0.4, 0.1), (0.4, 0.1), (0.0, 0.45)] {
                items.push(Placed::Shape {
                    rect: Rect::new(cx + dx * sz - sz * 0.5, cy + dy * sz - sz * 0.5, sz, sz),
                    kind: ShapeKind::Ellipse,
                    fill: Some(bottom_petals),
                    stroke: Some(Rgb(0x90, 0x60, 0x00)),
                    stroke_width: 0.5,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Dark Velvet Eye in center
            items.push(Placed::Shape {
                rect: Rect::new(cx - sz * 0.3, cy - sz * 0.15, sz * 0.6, sz * 0.5),
                kind: ShapeKind::Ellipse,
                fill: Some(eye_color),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersRedRose => {
            // Red Rose: Rich crimson rose blossom surrounded by deep green leaf sepals
            let rose_red = user_color.unwrap_or(Rgb(0xD0, 0x10, 0x20));
            let dark_red = Rgb(0x80, 0x00, 0x10);
            let leaf_green = Rgb(0x20, 0x60, 0x20);

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0;
            let r = rect.w.min(rect.h) * 0.38;

            // Leaves / Sepals
            for (dx, dy) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
                items.push(Placed::Shape {
                    rect: Rect::new(cx + dx * r - r * 0.3, cy + dy * r - r * 0.3, r * 0.6, r * 0.6),
                    kind: ShapeKind::Diamond,
                    fill: Some(leaf_green),
                    stroke: None,
                    stroke_width: 0.0,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }

            // Outer Petal layer
            items.push(Placed::Shape {
                rect: Rect::new(cx - r, cy - r, r * 2.0, r * 2.0),
                kind: ShapeKind::Ellipse,
                fill: Some(rose_red),
                stroke: Some(dark_red),
                stroke_width: 1.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Inner Petal swirl layer 1
            items.push(Placed::Shape {
                rect: Rect::new(cx - r * 0.65, cy - r * 0.55, r * 1.3, r * 1.1),
                kind: ShapeKind::Ellipse,
                fill: Some(dark_red),
                stroke: Some(Rgb(0xFF, 0x60, 0x70)),
                stroke_width: 0.75,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Inner Petal swirl layer 2
            items.push(Placed::Shape {
                rect: Rect::new(cx - r * 0.35, cy - r * 0.35, r * 0.7, r * 0.7),
                kind: ShapeKind::Ellipse,
                fill: Some(rose_red),
                stroke: Some(Rgb(0xFF, 0x80, 0x90)),
                stroke_width: 0.5,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersRoses => {
            // Roses: Twin rose buds connected by leafy trailing vines
            let rose_color = user_color.unwrap_or(Rgb(0xE0, 0x40, 0x60));
            let vine_color = Rgb(0x40, 0x80, 0x30);

            let cy = rect.y + rect.h / 2.0;
            let sz = rect.w.min(rect.h) * 0.28;

            // Connecting Vine line
            items.push(Placed::Shape {
                rect: Rect::new(rect.x, cy - 1.0, rect.w, 2.0),
                kind: ShapeKind::Rectangle,
                fill: Some(vine_color),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Two Rose Blossoms along the tile
            for cx in [rect.x + rect.w * 0.3, rect.x + rect.w * 0.7] {
                // Leaf sepals
                items.push(Placed::Shape {
                    rect: Rect::new(cx - sz * 0.6, cy - sz * 0.6, sz * 1.2, sz * 1.2),
                    kind: ShapeKind::Diamond,
                    fill: Some(vine_color),
                    stroke: None,
                    stroke_width: 0.0,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });

                // Rose bloom
                items.push(Placed::Shape {
                    rect: Rect::new(cx - sz * 0.7, cy - sz * 0.7, sz * 1.4, sz * 1.4),
                    kind: ShapeKind::Ellipse,
                    fill: Some(rose_color),
                    stroke: Some(Rgb(0x90, 0x10, 0x30)),
                    stroke_width: 0.75,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }
        }

        ArtStyle::WdArtFlowersTeacup => {
            // Teacup: Charming teacup with floral accent / steam motif
            let cup_color = user_color.unwrap_or(Rgb(0x30, 0x80, 0xB0)); // Porcelain Blue/Teal
            let floral_accent = Rgb(0xE0, 0x60, 0x80);
            let saucer_color = Rgb(0x80, 0xB0, 0xD0);

            let cx = rect.x + rect.w / 2.0;
            let cy = rect.y + rect.h / 2.0 + rect.h * 0.05;
            let w_cup = rect.w * 0.5;
            let h_cup = rect.h * 0.35;

            // Saucer
            items.push(Placed::Shape {
                rect: Rect::new(cx - w_cup * 0.7, cy + h_cup * 0.4, w_cup * 1.4, h_cup * 0.25),
                kind: ShapeKind::RoundedRectangle,
                fill: Some(saucer_color),
                stroke: Some(Rgb(0x20, 0x50, 0x80)),
                stroke_width: 0.75,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Teacup Body
            items.push(Placed::Shape {
                rect: Rect::new(cx - w_cup * 0.5, cy - h_cup * 0.5, w_cup, h_cup),
                kind: ShapeKind::RoundedRectangle,
                fill: Some(cup_color),
                stroke: Some(Rgb(0x10, 0x40, 0x70)),
                stroke_width: 1.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Teacup Handle
            items.push(Placed::Shape {
                rect: Rect::new(cx + w_cup * 0.4, cy - h_cup * 0.3, w_cup * 0.35, h_cup * 0.6),
                kind: ShapeKind::Ellipse,
                fill: None,
                stroke: Some(cup_color),
                stroke_width: 1.5,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // Floral emblem on teacup
            items.push(Placed::Shape {
                rect: Rect::new(cx - h_cup * 0.25, cy - h_cup * 0.25, h_cup * 0.5, h_cup * 0.5),
                kind: ShapeKind::Ellipse,
                fill: Some(floral_accent),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });
        }

        ArtStyle::WdArtFlowersTiny => {
            // Tiny Flowers: Continuous vine chain of delicate petite 5-petal blossoms
            let flower_color = user_color.unwrap_or(Rgb(0xF0, 0x70, 0x90));
            let center_color = Rgb(0xFF, 0xE0, 0x60);
            let vine_color = Rgb(0x60, 0xA0, 0x50);

            let cy = rect.y + rect.h / 2.0;

            // Continuous vine line
            items.push(Placed::Shape {
                rect: Rect::new(rect.x, cy - 0.75, rect.w, 1.5),
                kind: ShapeKind::Rectangle,
                fill: Some(vine_color),
                stroke: None,
                stroke_width: 0.0,
                freeform: None,
                effects: Default::default(),
                spin,
                extra: Default::default(),
                object: None,
            });

            // 3 Tiny Flowers per tile
            for i in 0..3 {
                let cx = rect.x + rect.w * (0.2 + 0.3 * i as f32);
                let r = rect.w.min(rect.h) * 0.12;

                // Petal ring
                items.push(Placed::Shape {
                    rect: Rect::new(cx - r * 1.2, cy - r * 1.2, r * 2.4, r * 2.4),
                    kind: ShapeKind::Ellipse,
                    fill: Some(flower_color),
                    stroke: None,
                    stroke_width: 0.0,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });

                // Center
                items.push(Placed::Shape {
                    rect: Rect::new(cx - r * 0.5, cy - r * 0.5, r, r),
                    kind: ShapeKind::Ellipse,
                    fill: Some(center_color),
                    stroke: None,
                    stroke_width: 0.0,
                    freeform: None,
                    effects: Default::default(),
                    spin,
                    extra: Default::default(),
                    object: None,
                });
            }
        }
    }
}
