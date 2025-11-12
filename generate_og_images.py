#!/usr/bin/env python3
"""
Generate Open Graph images for both commons-website and website.
Standard OG image size: 1200x630px
"""

from PIL import Image, ImageDraw, ImageFont
import os

# Bitcoin orange color
BITCOIN_ORANGE = "#F7931A"
DARK_GREY = "#4A4A4A"
WHITE = "#FFFFFF"

# Standard OG image dimensions
WIDTH = 1200
HEIGHT = 630

def create_og_image(title, subtitle, tagline, url, output_path_png, output_path_svg):
    """Create OG image with text-based design"""
    
    # Create PNG image
    img = Image.new('RGB', (WIDTH, HEIGHT), color=WHITE)
    draw = ImageDraw.Draw(img)
    
    # Try to use system fonts, fallback to default
    try:
        title_font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", 72)
        subtitle_font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans.ttf", 36)
        tagline_font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans.ttf", 28)
        url_font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans.ttf", 24)
    except:
        try:
            title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 72)
            subtitle_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 36)
            tagline_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 28)
            url_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 24)
        except:
            # Fallback to default font
            title_font = ImageFont.load_default()
            subtitle_font = ImageFont.load_default()
            tagline_font = ImageFont.load_default()
            url_font = ImageFont.load_default()
    
    # Calculate text positions (centered)
    y_start = 120
    
    # Draw title (orange)
    title_bbox = draw.textbbox((0, 0), title, font=title_font)
    title_width = title_bbox[2] - title_bbox[0]
    title_x = (WIDTH - title_width) // 2
    draw.text((title_x, y_start), title, fill=BITCOIN_ORANGE, font=title_font)
    
    # Draw subtitle (dark grey)
    y_subtitle = y_start + 100
    subtitle_bbox = draw.textbbox((0, 0), subtitle, font=subtitle_font)
    subtitle_width = subtitle_bbox[2] - subtitle_bbox[0]
    subtitle_x = (WIDTH - subtitle_width) // 2
    draw.text((subtitle_x, y_subtitle), subtitle, fill=DARK_GREY, font=subtitle_font)
    
    # Draw tagline (dark grey)
    y_tagline = y_subtitle + 60
    tagline_bbox = draw.textbbox((0, 0), tagline, font=tagline_font)
    tagline_width = tagline_bbox[2] - tagline_bbox[0]
    tagline_x = (WIDTH - tagline_width) // 2
    draw.text((tagline_x, y_tagline), tagline, fill=DARK_GREY, font=tagline_font)
    
    # Draw URL (dark grey, smaller)
    y_url = HEIGHT - 80
    url_bbox = draw.textbbox((0, 0), url, font=url_font)
    url_width = url_bbox[2] - url_bbox[0]
    url_x = (WIDTH - url_width) // 2
    draw.text((url_x, y_url), url, fill=DARK_GREY, font=url_font)
    
    # Save PNG
    img.save(output_path_png, 'PNG', optimize=True)
    print(f"Created PNG: {output_path_png}")
    
    # Create SVG
    svg_content = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg width="{WIDTH}" height="{HEIGHT}" xmlns="http://www.w3.org/2000/svg">
  <rect width="{WIDTH}" height="{HEIGHT}" fill="{WHITE}"/>
  <text x="{WIDTH // 2}" y="180" font-family="system-ui, -apple-system, sans-serif" font-size="72" font-weight="bold" fill="{BITCOIN_ORANGE}" text-anchor="middle">{title}</text>
  <text x="{WIDTH // 2}" y="280" font-family="system-ui, -apple-system, sans-serif" font-size="36" fill="{DARK_GREY}" text-anchor="middle">{subtitle}</text>
  <text x="{WIDTH // 2}" y="340" font-family="system-ui, -apple-system, sans-serif" font-size="28" fill="{DARK_GREY}" text-anchor="middle">{tagline}</text>
  <text x="{WIDTH // 2}" y="570" font-family="system-ui, -apple-system, sans-serif" font-size="24" fill="{DARK_GREY}" text-anchor="middle">{url}</text>
</svg>'''
    
    with open(output_path_svg, 'w') as f:
        f.write(svg_content)
    print(f"Created SVG: {output_path_svg}")

def main():
    # Create commons-website og-image
    commons_dir = "commons-website/assets"
    os.makedirs(commons_dir, exist_ok=True)
    
    create_og_image(
        title="The Bitcoin Commons",
        subtitle="Coordination Without Authority",
        tagline="A forkable governance model for Bitcoin implementations",
        url="thebitcoincommons.org",
        output_path_png=os.path.join(commons_dir, "og-image.png"),
        output_path_svg=os.path.join(commons_dir, "og-image.svg")
    )
    
    # Create website (BTCDecoded) og-image
    website_dir = "website/assets"
    os.makedirs(website_dir, exist_ok=True)
    
    create_og_image(
        title="BTCDecoded",
        subtitle="Bitcoin Decoded: Bitcoin LLVM and Cryptographic Commons",
        tagline="Bitcoin governance solved through mathematical specification",
        url="btcdecoded.org",
        output_path_png=os.path.join(website_dir, "og-image.png"),
        output_path_svg=os.path.join(website_dir, "og-image.svg")
    )
    
    print("\n✅ All OG images created successfully!")

if __name__ == "__main__":
    main()


