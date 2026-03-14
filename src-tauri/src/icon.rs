/// Generate an RGBA battery icon (22x22) representing a percentage.
/// percent < 0 means "unknown" (empty gray battery).
/// Colors: green > 50%, yellow 20-50%, red < 20%, gray = unknown.
pub fn generate_battery_icon(percent: f64, width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    // Canvas size is typically 22x22 on macOS.
    // We want the icon to look centered and sleek.
    let body_w = 18.0;
    let body_h = 10.0;
    let body_x = (width as f64 - body_w - 2.0) / 2.0; 
    let body_y = (height as f64 - body_h) / 2.0;
    
    let nub_w = 2.0;
    let nub_h = 4.0;
    let nub_x = body_x + body_w;
    let nub_y = body_y + (body_h - nub_h) / 2.0;
    
    // Fill level config
    let border_thickness = 1.0;
    let gap_thickness = 1.0;
    let r_border = 1.5;
    let r_nub = 1.0;
    
    let fill_max_w = body_w - (border_thickness + gap_thickness) * 2.0; 
    let fill_start_x = body_x + border_thickness + gap_thickness;
    
    let fill_w = if percent >= 0.0 {
        (percent / 100.0) * fill_max_w
    } else {
        0.0 // Unknown/Empty
    };

    let (fr, fg, fb) = if percent < 0.0 {
        (142.0, 142.0, 147.0) // Gray
    } else if percent > 50.0 {
        (52.0, 199.0, 89.0)  // Green
    } else if percent > 20.0 {
        (255.0, 204.0, 0.0)  // Yellow
    } else {
        (255.0, 59.0, 48.0)  // Red
    };

    // Subpixel grid for smooth anti-aliasing
    let grid_size = 4;
    let step = 1.0 / grid_size as f64;
    
    for y in 0..height {
        for x in 0..width {
            let px = x as f64;
            let py = y as f64;
            let idx = ((y * width + x) * 4) as usize;
            
            let mut border_hits = 0;
            let mut fill_hits = 0;
            let mut nub_hits = 0;
            let mut bg_fill_hits = 0;
            
            for i in 0..grid_size {
                for j in 0..grid_size {
                    let sx = px + (i as f64 + 0.5) * step;
                    let sy = py + (j as f64 + 0.5) * step;
                    
                    // Box SDF
                    let cx = body_x + body_w / 2.0;
                    let cy = body_y + body_h / 2.0;
                    let dx_b = (sx - cx).abs() - (body_w / 2.0 - r_border);
                    let dy_b = (sy - cy).abs() - (body_h / 2.0 - r_border);
                    let dist_body = (dx_b.max(0.0).powi(2) + dy_b.max(0.0).powi(2)).sqrt() + dx_b.max(dy_b).min(0.0) - r_border;
                    
                    // Nub SDF
                    let ncx = nub_x + nub_w / 2.0;
                    let ncy = nub_y + nub_h / 2.0;
                    let dx_n = (sx - ncx).abs() - (nub_w / 2.0 - r_nub);
                    let dy_n = (sy - ncy).abs() - (nub_h / 2.0 - r_nub);
                    let dist_nub = (dx_n.max(0.0).powi(2) + dy_n.max(0.0).powi(2)).sqrt() + dx_n.max(dy_n).min(0.0) - r_nub;
                    
                    let inside_body = dist_body <= 0.0;
                    let inside_inner = dist_body <= -border_thickness;
                    let is_body_border = inside_body && !inside_inner;
                    
                    // Prevent nub clipping into body styling
                    let is_nub = dist_nub <= 0.0 && !inside_body;
                    
                    if is_body_border { border_hits += 1; }
                    if is_nub { nub_hits += 1; }
                    
                    let fill_area_dist = - (border_thickness + gap_thickness);
                    if dist_body <= fill_area_dist {
                        if sx <= fill_start_x + fill_w {
                            fill_hits += 1;
                        } else {
                            bg_fill_hits += 1;
                        }
                    }
                }
            }
            
            let total_samples = (grid_size * grid_size) as f32;
            let alpha_border = border_hits as f32 / total_samples;
            let alpha_nub = nub_hits as f32 / total_samples;
            let alpha_fill = fill_hits as f32 / total_samples;
            let alpha_bg = bg_fill_hits as f32 / total_samples;
            
            let mut r = 0f32;
            let mut g = 0f32;
            let mut b = 0f32;
            let mut a = 0f32;
            
            // Draw background fill (dim)
            if alpha_bg > 0.0 {
                let na = alpha_bg * 0.15;
                if na > 0.0 {
                    r = 120.0; g = 120.0; b = 120.0; a = na;
                }
            }
            
            // Draw active fill
            if alpha_fill > 0.0 {
                let na = alpha_fill;
                if na > 0.0 {
                    let out_a = na + a * (1.0 - na);
                    if out_a > 0.0 {
                        r = (fr * na + r * a * (1.0 - na)) / out_a;
                        g = (fg * na + g * a * (1.0 - na)) / out_a;
                        b = (fb * na + b * a * (1.0 - na)) / out_a;
                    }
                    a = out_a;
                }
            }
            
            // Draw border & nub
            let border_total_a = alpha_border + alpha_nub;
            if border_total_a > 0.0 {
                let na = border_total_a.min(1.0);
                if na > 0.0 {
                    let br = 200.0;
                    let bg = 200.0;
                    let bb = 200.0;
                    let out_a = na + a * (1.0 - na);
                    if out_a > 0.0 {
                        r = (br * na + r * a * (1.0 - na)) / out_a;
                        g = (bg * na + g * a * (1.0 - na)) / out_a;
                        b = (bb * na + b * a * (1.0 - na)) / out_a;
                    }
                    a = out_a;
                }
            }
            
            pixels[idx] = r.clamp(0.0, 255.0).round() as u8;
            pixels[idx+1] = g.clamp(0.0, 255.0).round() as u8;
            pixels[idx+2] = b.clamp(0.0, 255.0).round() as u8;
            pixels[idx+3] = (a * 255.0).clamp(0.0, 255.0).round() as u8;
        }
    }

    Ok(pixels)
}
