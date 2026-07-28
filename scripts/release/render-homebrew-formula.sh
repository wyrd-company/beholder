#!/usr/bin/env bash

set -euo pipefail

version="${1:?version is required}"
artifact_directory="${2:?artifact directory is required}"
output="${3:?output path is required}"
repository="wyrd-company/beholder"

linux_x86_64_archive="$artifact_directory/beholder-linux-x86_64.tar.gz"
linux_arm64_archive="$artifact_directory/beholder-linux-arm64.tar.gz"
macos_arm64_archive="$artifact_directory/beholder-macos-arm64.tar.gz"

for archive in "$linux_x86_64_archive" "$linux_arm64_archive" "$macos_arm64_archive"; do
  if [[ ! -f "$archive" ]]; then
    echo "Missing expected Homebrew release asset: $archive" >&2
    exit 1
  fi
done

linux_x86_64_sha256="$(sha256sum "$linux_x86_64_archive" | cut -d' ' -f1)"
linux_arm64_sha256="$(sha256sum "$linux_arm64_archive" | cut -d' ' -f1)"
macos_arm64_sha256="$(sha256sum "$macos_arm64_archive" | cut -d' ' -f1)"

cat > "$output" <<FORMULA
class Beholder < Formula
  desc "Structural code index: symbols, complexity, and change-scoped risk"
  homepage "https://github.com/$repository"
  license "MIT"
  version "$version"

  on_macos do
    on_arm do
      url "https://github.com/$repository/releases/download/$version/beholder-macos-arm64.tar.gz"
      sha256 "$macos_arm64_sha256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/$repository/releases/download/$version/beholder-linux-arm64.tar.gz"
      sha256 "$linux_arm64_sha256"
    end

    on_intel do
      url "https://github.com/$repository/releases/download/$version/beholder-linux-x86_64.tar.gz"
      sha256 "$linux_x86_64_sha256"
    end
  end

  def install
    bin.install "beholder"
    prefix.install_metafiles
  end

  test do
    assert_match "structural code index", shell_output("#{bin}/beholder --help")
  end
end
FORMULA
