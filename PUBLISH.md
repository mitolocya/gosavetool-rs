# crates.io'da (Rust) yayınlama

Durum: kaynak hazır (kütüphane + `gosavetool` komut satırı aracı, birim testleri ve belge örnekleri dahil), ancak bu bilgisayarda Rust kurulu değil ve dış araç indirmek kapalıydı; bu yüzden **derlenmedi ve testleri çalıştırılmadı**. Mantık, test edilmiş Python/Java/Perl sürümleriyle aynı adımları izliyor. Yayından önce mutlaka:

```
winget install Rustlang.Rustup
rustup default stable
cargo test
cargo clippy -- -D warnings
cargo package --list
```

`cargo test` 6 birim testi ve 3 belge örneğini çalıştırır; hepsi geçmeli. Derleme hatası çıkarsa hata mesajı satırı gösterir; genelde tek satırlık tür düzeltmesidir.

## 1. Hesap

1. https://crates.io → **Log in with GitHub** (mevcut `mitolocya` hesabı).
2. **Account Settings** → e-posta ekle ve doğrula (yayın için zorunlu).

## 2. Yayınlama anahtarı

crates.io komut satırından yükleme için bir API anahtarı ister; anahtar sadece senin bilgisayarında kalmalı:

1. **Account Settings → API Tokens → New Token** → ad `gosavetool`, kapsam: `publish-new` ve `publish-update`, crate adı deseni `gosavetool`.
2. Bilgisayarda: `cargo login` → anahtarı yapıştır (Cargo onu `%USERPROFILE%\.cargo\credentials.toml` dosyasına kaydeder).

## 3. Yayınlama

Bu klasörde:

```
cargo publish --dry-run
cargo publish
```

Birkaç dakika içinde https://crates.io/crates/gosavetool sayfası açılır; belgeler https://docs.rs/gosavetool adresinde otomatik derlenir. `cargo install gosavetool` ile CLI kurulabilir.

İsteğe bağlı GitHub deposu: `mitolocya/gosavetool-rs` (`Cargo.toml` içindeki `repository`). Depo açılmayacaksa bu satırı silip öyle yayınla.

## Listeleme metinleri

Sayfa `Cargo.toml` ve README.md'den oluşur:

- **Crate adı:** gosavetool
- **Açıklama:** Parse, convert, shift and repair SRT and WebVTT subtitles; word count, social media character limits, Title Case and slugs. No dependencies, with a CLI.
- **Anahtar kelimeler (en fazla 5):** subtitles, srt, webvtt, word-count, slug
- **Kategoriler:** text-processing, multimedia, command-line-utilities
- **Ana sayfa:** https://www.gosavetool.com/
- **Uzun açıklama:** README.md

## Sonraki sürümler

`Cargo.toml` içinde `version` değerini artır, `cargo test`, `cargo publish`. Yayınlanan bir sürüm silinemez, sadece `cargo yank` ile geri çekilebilir.
