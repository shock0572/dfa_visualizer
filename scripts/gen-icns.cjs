const { execSync } = require("child_process");
const { existsSync, mkdirSync, rmSync } = require("fs");
const { join } = require("path");
const os = require("os");

if (os.platform() !== "darwin") process.exit(0);

const icons = join(__dirname, "..", "src-tauri", "icons");
const src = join(icons, "icon.png");
const out = join(icons, "icon.icns");
const iconset = join(icons, "icon.iconset");

if (existsSync(out)) process.exit(0);
if (!existsSync(src)) {
  console.log("Warning: icon.png not found, skipping .icns generation");
  process.exit(0);
}

mkdirSync(iconset, { recursive: true });

const sizes = [
  [16, "icon_16x16.png"],
  [32, "icon_16x16@2x.png"],
  [32, "icon_32x32.png"],
  [64, "icon_32x32@2x.png"],
  [128, "icon_128x128.png"],
  [256, "icon_128x128@2x.png"],
  [256, "icon_256x256.png"],
  [512, "icon_256x256@2x.png"],
  [512, "icon_512x512.png"],
  [1024, "icon_512x512@2x.png"],
];

for (const [size, name] of sizes) {
  execSync(`sips -z ${size} ${size} "${src}" --out "${join(iconset, name)}"`, {
    stdio: "ignore",
  });
}

execSync(`iconutil -c icns "${iconset}" -o "${out}"`);
rmSync(iconset, { recursive: true, force: true });
console.log("Generated", out);
