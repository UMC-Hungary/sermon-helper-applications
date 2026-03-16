// LucidGlass Shader v1.2
// Created by Andi Stone (Andilippi) - https://andilippi.co.uk

// Based on the shader 'Apple liquid glass replicate' by 4eckme - https://www.shadertoy.com/view/3cdXDX


// --- INFO: Usage Notes ---
uniform string usage_notes <
    string widget_type = "info";
    string label = "How to Use";
    string group = "Information";
> = "Add this shader to a source or scene you would like to apply a glass effect to. Adjust the settings to control the shape, position, and visual effects of the glass.";

uniform string usage_break <
    string widget_type = "info";
    string label = " ";
    string group = "Information";
> = " ";

uniform string creator_notes <
    string widget_type = "info";
    string label = "Author";
    string group = "Information";
> = "LucidGlass Shader v1.2 by Andi Stone (Andilippi)";

uniform string creator_break <
    string widget_type = "info";
    string label = " ";
> = " ";

// =============================================================================
// GLOBAL SETTINGS
// =============================================================================

uniform float LensPositionX <
    string label = "Position X";
    string tooltip = "Controls the horizontal position (X-axis) of the lens effect.";
    string widget_type = "slider";
    string group = "Global Settings";
    float minimum=-10000.0;
    float maximum=10000.0;
    float step=1.0;
> = 960.0;

uniform float LensPositionY <
    string label = "Position Y";
    string tooltip = "Controls the vertical position (Y-axis) of the lens effect.";
    string widget_type = "slider";
    string group = "Global Settings";
    float minimum=-10000.0;
    float maximum=10000.0;
    float step=1.0;
> = 540.0;

uniform bool TransparentBackground = true;

// =============================================================================
// SHAPE DEFINITION
// =============================================================================
uniform float shape_width <
    string label = "Shape Width (px)";
    string tooltip = "Defines the width of the glass shape in pixels.";
    string widget_type = "slider";
    string group = "Shape Definition";
    float minimum = 0.0;
    float maximum = 4096.0;
    float step = 1.0;
> = 400.0;

uniform float shape_height <
    string label = "Shape Height (px)";
    string tooltip = "Defines the height of the glass shape in pixels.";
    string widget_type = "slider";
    string group = "Shape Definition";
    float minimum = 0.0;
    float maximum = 4096.0;
    float step = 1.0;
> = 400.0;

uniform float corner_radius <
    string label = "Corner Radius (px)";
    string tooltip = "Defines the radius of the corners in pixels.";
    string widget_type = "slider";
    string group = "Shape Definition";
    float minimum = 0.0;
    float maximum = 1024.0;
    float step = 1.0;
> = 150.0;

uniform float squircleness <
    string label = "Squircleness (n)";
    string tooltip = "Controls the shape from a perfect circle (n=2) toward a square (higher n).";
    string widget_type = "slider";
    string group = "Shape Definition";
    float minimum = 0.01;
    float maximum = 100.0;
    float step = 0.01;
> = 3.0;

uniform float FeatheringPx <
    string label = "Edge Feathering (px)";
    string tooltip = "Width of the smooth edge transition. 0 for a hard edge.";
    string widget_type = "slider";
    string group = "Shape Definition";
    float minimum = 0.0;
    float maximum = 50.0;
    float step = 0.01;
> = 1.5;

// =============================================================================
// GLASS EFFECTS
// =============================================================================

uniform float BlurLevel <
    string label = "Blur Level";
    string tooltip = "Controls the amount of blur applied to the glass effect. Higher values increase blur.";
    string widget_type = "slider";
    string group = "Glass Effects";
    float minimum=0.0;
    float maximum=10.0;
    float step=0.01;
> = 2.5;

uniform float FrostStrength <
    string label = "Frost Strength";
    string tooltip = "Controls the intensity of the frosted glass noise. 0 for smooth blur, higher values for more frost.";
    string widget_type = "slider";
    string group = "Glass Effects";
    float minimum = 0.0;
    float maximum = 20.0;
    float step = 0.01;
> = 0.2;

uniform float4 TintColor = {1.0, 1.0, 1.0, 1.0};

uniform float TintStrength <
    string label = "Tint Strength";
    string tooltip = "Overall strength of the tint effect (0=none, 1=full based on Tint Colour setting).";
    string widget_type = "slider";
    string group = "Glass Effects";
    float minimum = 0.0;
    float maximum = 1.0;
    float step = 0.01;
> = 0.15;

// =============================================================================
// LENS DISTORTION EFFECTS
// =============================================================================

uniform float DistortionEdgeThicknessPx <
    string label = "Distortion Edge Thickness (px)";
    string widget_type = "slider";
    string group = "Lens Distortion";
    float minimum = 0.0;
    float maximum = 200.0;
    float step = 1.0;
> = 60.0;

uniform float MaxDistortionAmount <
    string label = "Max Distortion Amount";
    string tooltip = "Strength/direction of distortion. >0 pinches inwards, <0 bulges outwards.";
    string widget_type = "slider";
    string group = "Lens Distortion";
    float minimum = -2.0;
    float maximum = 2.0;
    float step = 0.01;
> = 0.8;

uniform float DistortionFalloffPower <
    string label = "Distortion Falloff Power";
    string tooltip = "Controls sharpness of distortion fade. >1 sharper edge, <1 softer edge.";
    string widget_type = "slider";
    string group = "Lens Distortion";
    float minimum = 0.1;
    float maximum = 10.0;
    float step = 0.01;
> = 2.0;

uniform float MagnificationAmount <
    string label = "Magnification Amount";
    string tooltip = "Controls zoom level of content through the glass. 1.0=normal, >1.0=zoomed in (magnified), <1.0=zoomed out.";
    string widget_type = "slider";
    string group = "Lens Distortion";
    float minimum = 0.1;
    float maximum = 5.0;
    float step = 0.01;
> = 2.0;

// =============================================================================
// OUTER GLOW EFFECTS
// =============================================================================

uniform bool EnableGlow = true;

uniform float4 GlowColor = {0.8, 0.9, 1.0, 0.8};

uniform float GlowBaseSpreadPx <
    string label = "Outer Glow Spread (px)";
    string tooltip = "Controls how far the outer glow extends from the edge of the shape.";
    string widget_type = "slider";
    string group = "Outer Glow Effects";
    float minimum = 0.0;
    float maximum = 100.0;
    float step = 1.0;
> = 5.0;

uniform float GlowFalloffPower <
    string label = "Outer Glow Falloff Power";
    string tooltip = "Controls sharpness of outer glow fade. >1 sharper, <1 softer.";
    string widget_type = "slider";
    string group = "Outer Glow Effects";
    float minimum = 0.0;
    float maximum = 10.0;
    float step = 0.01;
> = 1.0;

uniform bool EnableDirectionalGlow;

uniform float GlowDirectionAngle <
    string label = "Glow Direction (degrees)";
    string tooltip = "Direction of the glow in degrees. 0=right, 90=up, 180=left, 270=down.";
    string widget_type = "slider";
    string group = "Outer Glow Effects";
    float minimum = 0.0;
    float maximum = 360.0;
    float step = 1.0;
> = 0.0;

uniform float GlowDirectionalSpread <
    string label = "Directional Spread (degrees)";
    string tooltip = "How wide the directional glow spreads. 180=full sides, 90=quarter sides, 45=narrow beam.";
    string widget_type = "slider";
    string group = "Outer Glow Effects";
    float minimum = 10.0;
    float maximum = 180.0;
    float step = 1.0;
> = 180.0;

uniform float GlowDirectionalSoftness <
    string label = "Directional Softness";
    string tooltip = "How soft the edges of the directional glow are. 0=hard edges, 1=very soft.";
    string widget_type = "slider";
    string group = "Outer Glow Effects";
    float minimum = 0.0;
    float maximum = 1.0;
    float step = 0.01;
> = 0.5;



// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

// Optimise random function using fewer operations
float rand(float2 p) {
    return frac(sin(dot(p, float2(12.9898, 78.233))) * 43758.5453);
}

// Optimise mirror function using conditional instead of fmod
float mirror_val(float val) {
    val = abs(val);
    float mod_val = val - floor(val * 0.5) * 2.0;
    return mod_val > 1.0 ? (2.0 - mod_val) : mod_val;
}

float2 mirror_uv(float2 uv_in) {
    return float2(mirror_val(uv_in.x), mirror_val(uv_in.y));
}

// Optimise frost function with early exit and cached calculations
float2 apply_frost_to_uv(float2 input_uv, float frost_strength, float2 tex_uv_size) {
    if (frost_strength <= 0.001) {
        return input_uv;
    }
    
    // Pre-calculate frost multiplier
    float frost_multiplier = frost_strength / max(tex_uv_size.x, tex_uv_size.y);
    
    // Simplified random with fewer operations
    float2 p = input_uv * 1000.0;
    float rand_x = frac(sin(dot(p, float2(12.9898, 78.233))) * 43758.5453);
    float rand_y = frac(sin(dot(p, float2(78.233, 12.9898))) * 43758.5453);
    
    float2 random_offset = (float2(rand_x, rand_y) - 0.5) * frost_multiplier;
    return input_uv + random_offset;
}

// =============================================================================
// MAIN SHADER FUNCTION
// =============================================================================

float4 mainImage(VertData v_in) : TARGET
{
    float2 uv = v_in.uv;
    float2 mouse = float2(LensPositionX, LensPositionY);

    // Cache expensive calculations
    float2 current_abs_px = uv * uv_size;
    float2 p_squircle = current_abs_px - mouse;
    
    // Pre-calculate commonly used values
    float2 shape_dim_px = float2(max(shape_width, 1.0), max(shape_height, 1.0));
    float R_squircle = max(corner_radius, 0.0);
    float n_squircle = max(squircleness, 0.01);
    float2 half_shape_dim = 0.5 * shape_dim_px;

    // =========================================================================
    // SQUIRCLE SHAPE CALCULATION
    // =========================================================================
    
    float sdf;
    if (R_squircle < 0.001) {
        // Simple rectangle case
        float2 d = abs(p_squircle) - half_shape_dim;
        sdf = length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
    } else {
        // Rounded rectangle case
        float2 dPos = abs(p_squircle) - (half_shape_dim - R_squircle);
        float inside_check = min(max(dPos.x, dPos.y), 0.0);
        dPos = max(dPos, 0.0);
        float2 u_squircle = dPos / R_squircle;
        
        // Optimise power calculations
        float ux_pow = pow(abs(u_squircle.x), n_squircle);
        float uy_pow = pow(abs(u_squircle.y), n_squircle);
        float sup = pow(ux_pow + uy_pow, 1.0 / n_squircle);
        sdf = inside_check + R_squircle * (sup - 1.0);
    }

    // Optimise feathering calculation
    float current_FeatheringPx = max(FeatheringPx, 0.0);
    float effect_alpha;
    if (current_FeatheringPx < 0.01) {
        effect_alpha = sdf < 0.0 ? 1.0 : 0.0;
    } else {
        effect_alpha = saturate(0.5 - sdf / current_FeatheringPx);
    }    // Early exit for pixels outside the effect - but only if outer glow is disabled
    if (effect_alpha < 0.001) { 
        // Check if we should process outer glow
        bool should_process_outer_glow = EnableGlow && GlowBaseSpreadPx > 0.001 && GlowColor.a > 0.001 && sdf > 0.0;
        
        if (!should_process_outer_glow) {
            if (TransparentBackground) { 
                return float4(0.0, 0.0, 0.0, 0.0); 
            } else {
                return image.Sample(textureSampler, uv); 
            }
        }
    }
      
    // =========================================================================
    // EFFECT RGB COLOR CALCULATION
    // =========================================================================
    
    float3 effect_rgb_calculated;
    {        
        // =====================================================================
        // LENS DISTORTION & MAGNIFICATION CALCULATION
        // =====================================================================
        
        float dist_from_edge_px = -sdf;
        float current_DistortionEdgeThicknessPx = max(DistortionEdgeThicknessPx, 0.01);
        float distortion_ramp = saturate(1.0 - (dist_from_edge_px / current_DistortionEdgeThicknessPx));
        float distortion_effect_intensity = pow(distortion_ramp, max(DistortionFalloffPower, 0.01));
        
        // Combine distortion with magnification
        float distortion_zoom = 1.0 - distortion_effect_intensity * MaxDistortionAmount;
        float magnification_zoom = 1.0 / max(MagnificationAmount, 0.1); // Invert for correct magnification behavior
        float lens_zoom = distortion_zoom * magnification_zoom;
        lens_zoom = clamp(lens_zoom, -5.0, 10.0); // Extended range to accommodate magnification
        
        float2 squircle_center_uv = mouse / uv_size;
        float2 lens_uv_from_squircle_center = uv - squircle_center_uv;
        float2 lens_sample_uv_raw = lens_uv_from_squircle_center * lens_zoom + squircle_center_uv;
        
        // =====================================================================
        // BLUR PROCESSING
        // =====================================================================
        
        float4 effect_accumulator = float4(0.0, 0.0, 0.0, 0.0);
        float total_blur_weight = 0.0;
        
        // Pre-calculate blur offset multiplier and frost check
        float2 blur_offset_multiplier = (0.5 * BlurLevel) / uv_size;
        bool use_frost = FrostStrength > 0.001;
        
        float weights[5] = {1.0, 2.0, 3.0, 2.0, 1.0};
        
        for (int x = 0; x < 5; x++) {
            for (int y = 0; y < 5; y++) {
                float2 blur_offset = float2(x - 2.0, y - 2.0) * blur_offset_multiplier;
                float current_weight = weights[x] * weights[y];
                
                float2 sample_uv = blur_offset + lens_sample_uv_raw;
                
                // Conditional frost application
                if (use_frost) {
                    sample_uv = apply_frost_to_uv(sample_uv, FrostStrength, uv_size);
                }
                
                effect_accumulator += image.Sample(textureSampler, mirror_uv(sample_uv)) * current_weight;
                total_blur_weight += current_weight;
            }
        }

        // Normalise the accumulated color
        if (total_blur_weight > 0.0001) {
            effect_accumulator /= total_blur_weight;
        } else {
            // Fallback sample
            float2 fallback_uv = use_frost ? 
                apply_frost_to_uv(lens_sample_uv_raw, FrostStrength, uv_size) : 
                lens_sample_uv_raw;
            effect_accumulator = image.Sample(textureSampler, mirror_uv(fallback_uv));
        }
        
        effect_rgb_calculated = effect_accumulator.rgb;
    }    
    
    // =========================================================================
    // TINTING
    // =========================================================================
    
    if (TintStrength > 0.001 && TintColor.a > 0.001) {
        float tint_factor = TintColor.a * TintStrength;
        effect_rgb_calculated = lerp(effect_rgb_calculated, TintColor.rgb, tint_factor);
    }    // =========================================================================
    // OUTER GLOW CALCULATION
    // =========================================================================
    
    if (EnableGlow && GlowBaseSpreadPx > 0.001 && GlowColor.a > 0.001 && effect_alpha > 0.001) {
        float current_GlowBaseSpreadPx = max(GlowBaseSpreadPx, 0.01);
        float effective_glow_spread = current_GlowBaseSpreadPx;


          // Glow intensity calculation
        float glow_intensity_factor = 0.0;
        if (sdf > 0.0) {
            float distance_outside_edge = sdf;
            float glow_distance_norm = saturate(1.0 - (distance_outside_edge / effective_glow_spread));
            glow_intensity_factor = pow(glow_distance_norm, max(GlowFalloffPower, 0.01));
        }        float actual_glow_alpha_component = glow_intensity_factor * GlowColor.a;
        if (actual_glow_alpha_component > 0.001) {
            // For outer glow, we need to handle it differently in the final composition
            // Store the glow information for later use
        }
    }    
    
    // =========================================================================
    // FINAL COLOUR COMPOSITION
    // =========================================================================
    
    float4 originalColor = image.Sample(textureSampler, uv);
      // Handle outer glow with anti-aliasing
    float outer_glow_alpha = 0.0;
    float3 outer_glow_color = float3(0.0, 0.0, 0.0);
      if (EnableGlow && GlowBaseSpreadPx > 0.001 && GlowColor.a > 0.001) {
        float effective_glow_spread = max(GlowBaseSpreadPx, 0.01);
        
        // Calculate glow for pixels outside or near the edge
        if (sdf > -1.0) { // Include a small transition zone inside the edge
            float distance_from_edge = max(sdf, 0.0); // Only positive distances contribute to glow
            float glow_distance_norm = saturate(1.0 - (distance_from_edge / effective_glow_spread));
            float glow_intensity_factor = pow(glow_distance_norm, max(GlowFalloffPower, 0.01));
            
            // Apply directional glow if enabled
            if (EnableDirectionalGlow) {
                // Calculate the direction from shape center to current pixel
                float2 shape_center = float2(LensPositionX, LensPositionY);
                float2 pixel_pos = float2(uv.x * uv_size.x, uv.y * uv_size.y);
                float2 direction_to_pixel = pixel_pos - shape_center;
                
                // Convert to angle (0-360 degrees)
                float pixel_angle = atan2(direction_to_pixel.y, direction_to_pixel.x) * 57.29577951308232; // 180/PI
                if (pixel_angle < 0.0) pixel_angle += 360.0;
                
                // Calculate target direction (opposite sides)
                float target_angle_1 = GlowDirectionAngle;
                float target_angle_2 = fmod(GlowDirectionAngle + 180.0, 360.0);
                
                // Calculate angular distances to both target directions
                float angle_diff_1 = abs(pixel_angle - target_angle_1);
                float angle_diff_2 = abs(pixel_angle - target_angle_2);
                
                // Handle wrap-around (e.g., 350° and 10° are close)
                if (angle_diff_1 > 180.0) angle_diff_1 = 360.0 - angle_diff_1;
                if (angle_diff_2 > 180.0) angle_diff_2 = 360.0 - angle_diff_2;
                
                // Use the smaller angle difference
                float min_angle_diff = min(angle_diff_1, angle_diff_2);
                
                // Calculate directional mask
                float half_spread = GlowDirectionalSpread * 0.5;
                float directional_mask = 0.0;
                
                if (min_angle_diff <= half_spread) {
                    if (GlowDirectionalSoftness > 0.001) {
                        // Soft transition
                        float softness_range = half_spread * GlowDirectionalSoftness;
                        float fade_start = half_spread - softness_range;
                        
                        if (min_angle_diff <= fade_start) {
                            directional_mask = 1.0;
                        } else {
                            float fade_progress = (min_angle_diff - fade_start) / softness_range;
                            directional_mask = 1.0 - smoothstep(0.0, 1.0, fade_progress);
                        }
                    } else {
                        // Hard transition
                        directional_mask = 1.0;
                    }
                }
                
                // Apply directional mask to glow intensity
                glow_intensity_factor *= directional_mask;
            }
            
            outer_glow_alpha = glow_intensity_factor * GlowColor.a;
            outer_glow_color = GlowColor.rgb;
        }
    }
    
    // Calculate smooth edge transition for anti-aliasing
    float edge_transition_width = 1.0; // 1 pixel transition zone
    float edge_blend_factor = saturate((sdf + edge_transition_width) / edge_transition_width);
      if (TransparentBackground) {
        // Smooth blend between glass effect and outer glow
        float3 glass_result = effect_rgb_calculated;
        float glass_alpha = effect_alpha * (1.0 - edge_blend_factor);
        
        float3 glow_result = outer_glow_color;
        float glow_alpha = outer_glow_alpha * edge_blend_factor;
        
        // Combine glass and glow with proper alpha blending
        float total_alpha = glass_alpha + glow_alpha * (1.0 - glass_alpha);
        float3 final_color = glass_alpha > 0.001 ? 
            (glass_result * glass_alpha + glow_result * glow_alpha * (1.0 - glass_alpha)) / max(total_alpha, 0.001) :
            glow_result;
            
        return float4(final_color, total_alpha);
    } else {
        // Smooth blend between glass effect and outer glow over original image
        float3 glass_result = lerp(originalColor.rgb, effect_rgb_calculated, effect_alpha * (1.0 - edge_blend_factor));
        float3 glow_result = lerp(originalColor.rgb, outer_glow_color, outer_glow_alpha * edge_blend_factor);
        
        // Blend the results based on distance from edge
        float3 final_rgb = lerp(glow_result, glass_result, 1.0 - edge_blend_factor);
        return float4(final_rgb, originalColor.a);
    }
}