import sharp from 'sharp';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const size = 1024;
const cornerRadius = size * 0.22; // ~22% corner radius (standard macOS)
const shadowOffset = 10;
const shadowBlur = 30;
const shadowOpacity = 0.3;

// Paths relative to script location
const inputPath = join(__dirname, '../../../images/radium-logo-square.png');
const outputPath = join(__dirname, '../app-icon.png');

console.log('Creating macOS-style icon...');
console.log(`Input: ${inputPath}`);
console.log(`Output: ${outputPath}`);

if (!existsSync(inputPath)) {
  console.error(`Error: Input file not found at ${inputPath}`);
  process.exit(1);
}

try {
  // Load and resize the source image
  const sourceImage = sharp(inputPath);
  const metadata = await sourceImage.metadata();
  
  console.log(`Source image: ${metadata.width}x${metadata.height}`);

  // Create rounded rectangle SVG mask
  const roundedRectSvg = Buffer.from(
    `<svg width="${size}" height="${size}" xmlns="http://www.w3.org/2000/svg">
      <rect x="0" y="0" width="${size}" height="${size}" 
            rx="${cornerRadius}" ry="${cornerRadius}" 
            fill="white"/>
    </svg>`
  );

  // Create shadow layer
  const shadowSize = size + shadowOffset * 2;
  const shadowSvg = Buffer.from(
    `<svg width="${shadowSize}" height="${shadowSize}" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <filter id="shadow">
          <feGaussianBlur in="SourceAlpha" stdDeviation="${shadowBlur / 2}"/>
        </filter>
      </defs>
      <rect x="${shadowOffset}" y="${shadowOffset}" 
            width="${size}" height="${size}" 
            rx="${cornerRadius}" ry="${cornerRadius}" 
            fill="black" 
            opacity="${shadowOpacity}"
            filter="url(#shadow)"/>
    </svg>`
  );

  // Process the image
  const resizedImage = await sourceImage
    .resize(size, size, { 
      fit: 'contain', 
      background: { r: 0, g: 0, b: 0, alpha: 0 } 
    })
    .toBuffer();

  // Create shadow
  const shadowBuffer = await sharp(shadowSvg)
    .resize(shadowSize, shadowSize)
    .png()
    .toBuffer();

  // Composite: shadow + image with rounded corners
  await sharp({
    create: {
      width: shadowSize,
      height: shadowSize,
      channels: 4,
      background: { r: 0, g: 0, b: 0, alpha: 0 }
    }
  })
    .composite([
      {
        input: shadowBuffer,
        blend: 'over'
      },
      {
        input: resizedImage,
        left: shadowOffset,
        top: shadowOffset,
        blend: 'over'
      },
      {
        input: roundedRectSvg,
        left: shadowOffset,
        top: shadowOffset,
        blend: 'dest-in'
      }
    ])
    .png()
    .toFile(outputPath);

  console.log(`✅ macOS icon created successfully at ${outputPath}`);
  console.log(`   Size: ${shadowSize}x${shadowSize}px`);
  console.log(`   Corner radius: ${cornerRadius}px`);
  console.log(`   Shadow: ${shadowOffset}px offset, ${shadowBlur}px blur`);
} catch (error) {
  console.error('Error creating icon:', error);
  process.exit(1);
}

