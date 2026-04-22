pkgname=vtopssh
pkgver=0.1.3
pkgrel=1
pkgdesc="Terminal UI system monitor for Linux with remote host support over SSH"
arch=("x86_64" "aarch64")
url="https://github.com/vrubelroman/vtopssh"
license=("MIT")
depends=("openssh" "iputils" "util-linux")
optdepends=("docker: Docker widget support")
makedepends=("cargo")
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=("SKIP")

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --locked
}

check() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo test --locked
}

package() {
  cd "$srcdir/$pkgname-$pkgver"

  install -Dm755 "target/release/vtopssh" "$pkgdir/usr/bin/vtopssh"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/vtopssh/README.md"
  install -Dm644 "assets/config.example.toml" \
    "$pkgdir/usr/share/doc/vtopssh/config.example.toml"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/vtopssh/LICENSE"
}
