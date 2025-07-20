#!/bin/bash
set -e

echo "🎨 Optimizing game images..."

# Change to project root
cd "$(dirname "$0")/.."

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick not found! Please install it:"
    echo "   brew install imagemagick"
    exit 1
fi

# Check if cwebp is installed for WebP conversion
if ! command -v cwebp &> /dev/null; then
    echo "❌ cwebp not found! Please install it:"
    echo "   brew install webp"
    exit 1
fi

# Create optimized assets directory
OPTIMIZED_DIR="assets/game-assets-optimized"
mkdir -p "$OPTIMIZED_DIR/sprites"
mkdir -p "$OPTIMIZED_DIR/backgrounds"

echo "📦 Optimizing sprites..."

# Optimize player sprites (resize to 64x64)
echo "  • Optimizing Ship_LVL_1.png..."
convert "assets/game-assets/sprites/Ship_LVL_1.png" \
    -resize 64x64 \
    -strip \
    -quality 95 \
    "$OPTIMIZED_DIR/sprites/Ship_LVL_1.png"

echo "  • Optimizing Ship_player_2.png..."
convert "assets/game-assets/sprites/Ship_player_2.png" \
    -resize 64x64 \
    -strip \
    -quality 95 \
    "$OPTIMIZED_DIR/sprites/Ship_player_2.png"

# Optimize boid sprite (resize to 32x32)
echo "  • Optimizing Ship_04.png..."
convert "assets/game-assets/sprites/Ship_04.png" \
    -resize 32x32 \
    -strip \
    -quality 95 \
    "$OPTIMIZED_DIR/sprites/Ship_04.png"

# Optimize projectile sprite (resize to 18x18)
echo "  • Optimizing laser1_small.png..."
convert "assets/game-assets/sprites/laser1_small.png" \
    -resize 18x18 \
    -strip \
    -interlace none \
    -quality 95 \
    "$OPTIMIZED_DIR/sprites/laser1_small.png"

echo "🏞️  Optimizing backgrounds..."

# Optimize background images (resize to 1024x768 with 85% quality)
for bg in derelict_ship_main.png derelict_ship_2.png derelict_ship_3.png; do
    echo "  • Optimizing $bg..."
    convert "assets/game-assets/backgrounds/$bg" \
        -resize 1024x768 \
        -strip \
        -quality 85 \
        "$OPTIMIZED_DIR/backgrounds/$bg"
done

echo "🌐 Creating WebP versions..."

# Convert all optimized PNGs to WebP
for png in $(find "$OPTIMIZED_DIR" -name "*.png"); do
    webp_file="${png%.png}.webp"
    echo "  • Converting $(basename $png) to WebP..."
    cwebp -q 85 "$png" -o "$webp_file" -quiet
done

# Calculate size savings
ORIGINAL_SIZE=$(du -sh assets/game-assets | cut -f1)
OPTIMIZED_SIZE=$(du -sh "$OPTIMIZED_DIR" | cut -f1)

echo ""
echo "✅ Optimization complete!"
echo "   Original size: $ORIGINAL_SIZE"
echo "   Optimized size: $OPTIMIZED_SIZE"
echo ""
echo "📁 Optimized assets saved to: $OPTIMIZED_DIR"
echo ""
echo "Next steps:"
echo "1. Test the optimized images in the game"
echo "2. If satisfied, replace the original assets:"
echo "   cp -r $OPTIMIZED_DIR/* assets/game-assets/"
echo "3. Update the client code to use WebP with PNG fallback"