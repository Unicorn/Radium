#!/usr/bin/env python3
"""
Create a macOS-style app icon with rounded corners and shadow.
Requires Pillow: pip install Pillow
"""
import sys
import os
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFilter
except ImportError:
    print("Error: Pillow is required. Install it with: pip install Pillow")
    sys.exit(1)

# Configuration
size = 1024
corner_radius = int(size * 0.22)  # ~22% corner radius (standard macOS)
shadow_offset = 10
shadow_blur = 30
shadow_opacity = 0.3

# Paths - resolve relative to project root
script_dir = Path(__file__).parent.resolve()
# From scripts/ -> desktop/ -> apps/ -> radium/ -> root/
project_root = script_dir / '../../../../'
input_path = project_root.resolve() / 'images/logo-apple.png'
output_path = script_dir.parent / 'app-icon.png'

print('Creating macOS-style icon...')
print(f'Input: {input_path}')
print(f'Output: {output_path}')

if not input_path.exists():
    print(f'Error: Input file not found at {input_path}')
    sys.exit(1)

try:
    # Load and resize source image
    img = Image.open(input_path)
    print(f'Source image: {img.size[0]}x{img.size[1]}')
    
    # Resize to target size while maintaining aspect ratio
    img = img.resize((size, size), Image.Resampling.LANCZOS)
    
    # Create shadow layer
    shadow_size = size + shadow_offset * 2
    shadow = Image.new('RGBA', (shadow_size, shadow_size), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    
    # Draw rounded rectangle for shadow
    shadow_draw.rounded_rectangle(
        [(shadow_offset, shadow_offset), 
         (size + shadow_offset, size + shadow_offset)],
        radius=corner_radius,
        fill=(0, 0, 0, int(255 * shadow_opacity))
    )
    
    # Apply blur to shadow
    shadow = shadow.filter(ImageFilter.GaussianBlur(radius=shadow_blur))
    
    # Create rounded corner mask
    mask = Image.new('L', (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle(
        [(0, 0), (size, size)],
        radius=corner_radius,
        fill=255
    )
    
    # Apply mask to image
    img_masked = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    img_masked.paste(img, (0, 0))
    img_masked.putalpha(mask)
    
    # Composite shadow + masked image
    result = Image.new('RGBA', (shadow_size, shadow_size), (0, 0, 0, 0))
    result.paste(shadow, (0, 0), shadow)
    result.paste(img_masked, (shadow_offset, shadow_offset), img_masked)
    
    # Save result
    result.save(output_path, 'PNG', optimize=True)
    
    print(f'✅ macOS icon created successfully at {output_path}')
    print(f'   Size: {shadow_size}x{shadow_size}px')
    print(f'   Corner radius: {corner_radius}px')
    print(f'   Shadow: {shadow_offset}px offset, {shadow_blur}px blur')
    
except Exception as error:
    print(f'Error creating icon: {error}')
    import traceback
    traceback.print_exc()
    sys.exit(1)

