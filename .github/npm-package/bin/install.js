const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');
const https = require('https');
const { createHash } = require('crypto');

const PKG_ROOT = path.join(__dirname, '..');
const BIN_DIR = path.join(PKG_ROOT, 'bin');
const NATIVE_DIR = path.join(BIN_DIR, '.rninja-binaries');
const ARCH_MAP = {
  'x64': 'x86_64',
  'arm64': 'aarch64'
};

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();
  const mappedArch = ARCH_MAP[arch] || arch;

  if (platform === 'win32') return { platform: 'pc-windows-msvc', arch: mappedArch, ext: 'zip' };
  if (platform === 'darwin') return { platform: 'apple-darwin', arch: mappedArch, ext: 'tar.gz' };
  if (platform === 'linux') return { platform: 'unknown-linux-gnu', arch: mappedArch, ext: 'tar.gz' };
  throw new Error(`Unsupported platform: ${platform}`);
}

function getDownloadUrl(version, platform, arch, ext) {
  return `https://github.com/neul-labs/rninja/releases/download/v${version}/rninja-${version}-${arch}-${platform}.${ext}`;
}

function getChecksumUrl(version) {
  return `https://github.com/neul-labs/rninja/releases/download/v${version}/checksums.txt`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        downloadFile(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      if (response.statusCode !== 200) {
        reject(new Error(`HTTP ${response.statusCode}: ${url}`));
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
  const version = require(path.join(PKG_ROOT, 'package.json')).version;
  const { platform, arch, ext } = getPlatform();

  console.log(`Installing rninja ${version} for ${platform} ${arch}`);

  const tarball = `rninja-${version}-${arch}-${platform}.${ext}`;
  const tarballPath = path.join(BIN_DIR, tarball);
  const url = getDownloadUrl(version, platform, arch, ext);

  console.log(`Downloading from ${url}`);

  // Ensure native directory exists
  if (!fs.existsSync(NATIVE_DIR)) {
    fs.mkdirSync(NATIVE_DIR, { recursive: true });
  }

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

    // Download tarball/zip
    await downloadFile(url, tarballPath);

    // Verify checksum
    console.log('Verifying checksum...');
    verifyChecksum(tarballPath, checksums);

    // Extract
    console.log('Extracting...');
    if (ext === 'zip') {
      execSync(`powershell -command "Expand-Archive -Path '${tarballPath}' -DestinationPath '${NATIVE_DIR}' -Force"`, { cwd: BIN_DIR });
    } else {
      execSync(`tar -xzf ${tarball} -C .rninja-binaries`, { cwd: BIN_DIR });
    }

    // Cleanup
    fs.unlinkSync(tarballPath);

    // Make binaries executable on Unix
    if (os.platform() !== 'win32') {
      for (const binary of ['rninja', 'rninja-cached', 'rninja-daemon']) {
        const binaryPath = path.join(NATIVE_DIR, binary);
        if (fs.existsSync(binaryPath)) {
          fs.chmodSync(binaryPath, '755');
        }
      }
    }

    console.log('Installation complete!');
  } catch (err) {
    console.error(`Installation failed: ${err.message}`);
    process.exit(1);
  }
}

install();
