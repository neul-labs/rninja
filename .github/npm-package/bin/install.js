const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');
const https = require('https');
const { createHash } = require('crypto');

const PKG_ROOT = __dirname;
const BIN_DIR = path.join(PKG_ROOT, 'bin');
const ARCH_MAP = {
  'x64': 'x86_64',
  'arm64': 'aarch64'
};

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();
  if (platform === 'win32') return { platform: 'pc-windows', arch: 'x86_64' };
  if (platform === 'darwin') return { platform: 'apple-darwin', arch };
  if (platform === 'linux') return { platform: 'unknown-linux-gnu', arch: ARCH_MAP[arch] || arch };
  throw new Error(`Unsupported platform: ${platform}`);
}

function getDownloadUrl(version, platform, arch) {
  return `https://github.com/dipankarsarkar/rninja/releases/download/v${version}/rninja-${version}-${arch}-${platform}.tar.gz`;
}

function getChecksumUrl(version) {
  return `https://github.com/dipankarsarkar/rninja/releases/download/v${version}/checksums.txt`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        downloadFile(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      response.pipe(file);
      file.on('finish', () => {
        file.close(resolve);
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

function verifyChecksum(filePath, checksums) {
  const fileContent = fs.readFileSync(filePath);
  const hash = createHash('sha256').update(fileContent).digest('hex');
  const basename = path.basename(filePath);
  if (!checksums[basename] && !checksums[`rninja/${basename}`]) {
    throw new Error(`No checksum found for ${basename}`);
  }
  const expected = checksums[basename] || checksums[`rninja/${basename}`];
  if (hash !== expected) {
    throw new Error(`Checksum mismatch for ${basename}: expected ${expected}, got ${hash}`);
  }
}

async function install() {
  const version = require('./package.json').version;
  const { platform, arch } = getPlatform();

  console.log(`Installing rninja ${version} for ${platform} ${arch}`);

  const tarball = `rninja-${version}-${arch}-${platform}.tar.gz`;
  const tarballPath = path.join(BIN_DIR, tarball);
  const url = getDownloadUrl(version, platform, arch);

  console.log(`Downloading from ${url}`);

  try {
    // Download checksums
    const checksumUrl = getChecksumUrl(version);
    console.log(`Downloading checksums from ${checksumUrl}`);
    const checksumData = await new Promise((resolve, reject) => {
      https.get(checksumUrl, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => resolve(data));
        res.on('error', reject);
      });
    });

    // Parse checksums
    const checksums = {};
    checksumData.split('\n').forEach(line => {
      const match = line.match(/^([a-f0-9]+)\s+\*?(.*)$/);
      if (match) {
        checksums[match[2]] = match[1];
      }
    });

    // Download tarball
    await downloadFile(url, tarballPath);

    // Verify checksum
    console.log('Verifying checksum...');
    verifyChecksum(tarballPath, checksums);

    // Extract
    console.log('Extracting...');
    execSync(`tar -xzf ${tarball} -C ${BIN_DIR}`, { cwd: BIN_DIR });

    // Cleanup
    fs.unlinkSync(tarballPath);

    // Make binaries executable
    const rninjaPath = path.join(BIN_DIR, 'rninja');
    const rninjaCachedPath = path.join(BIN_DIR, 'rninja-cached');
    if (fs.existsSync(rninjaPath)) fs.chmodSync(rninjaPath, '755');
    if (fs.existsSync(rninjaCachedPath)) fs.chmodSync(rninjaCachedPath, '755');

    console.log('Installation complete!');
  } catch (err) {
    console.error(`Installation failed: ${err.message}`);
    process.exit(1);
  }
}

install();
