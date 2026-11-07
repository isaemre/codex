# Makas
Bu repo openai/codex'in kurum içi fork'u. Amaç: Makas adında, sadece şirket içi LLM ile çalışan kurumsal bir kodlama agent'ı. Fork'u upstream'e yakın tutuyoruz ki yeni Codex sürümlerine kolay geçebilelim.
Vizyon: `makas/docs/VISION.md`. Kararlar: `makas/docs/KARARLAR.md`. Kararlar çatıştığında öncelik: güvenlik > sürdürülebilirlik > kullanıcı deneyimi > özellik.

## Temel kurallar
- Kendi Rust kodumuzu codex-rs/makas/ altına, diğer her şeyi (doküman, Electron) makas/ altına yaz.
- Upstream dosyalarında değişikliği minimumda tut ve değiştirdiğin yerin üstüne `// MAKAS: neden (K-xx)` yaz.
- Upstream kodunu silme; bir özelliği kapatman gerekiyorsa tek noktadan kapat.
- Önce config ile çözülüp çözülemeyeceğine bak.
- Kod Linux, macOS ve Windows'ta çalışmalı.
- Model bağlantısı sadece şirket içi LLM'e gitmeli. LLM adresi sadece `codex-makas-config`'teki derleme sabitinden gelir; yeni sağlayıcı, URL ya da dış ağ çağrısı ekleme.
- Upstream dosyasına dokunan işlerde önce plan çıkar ve onayımı al.

## Upstream'in çalışma şekli
- Bir klasörde AGENTS.md varsa önce onu oku.
- Değişiklikten sonra codex-rs içinde `just fmt` çalıştır; testler için `just test -p <crate>` kullan. Komutların var olduğunu justfile'dan doğrula.
- codex-core'a kod ekleme; yeni kodu kendi crate'inde yaz, core'da sadece tek satırlık kanca bırak (K-17).

## Dil
- Kod, yorum ve commit İngilizce, dokümanlar Türkçe. Push etme.
