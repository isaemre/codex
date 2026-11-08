# Makas karar kaydı

Her kararda iki şey yazılır: ne yapıyoruz, neden yapıyoruz.
Bir karar değişirse eskisi silinmez; altına "K-xx ile değiştirildi" notu düşülür ve yeni karar eklenir.

## Repo ve senkron
- **K-01 · Tek repo.** Codex fork'u ve Electron uygulaması (`makas/desktop/`) aynı repoda durur.
  - *Neden:* Upstream'de olmayan bir klasör merge'de çakışmaz; protokol değiştiğinde Rust tarafı ve arayüz aynı değişiklikte güncellenir.
- **K-02 · Senkron sadece release etiketlerinden.** Upstream'in `rust-vX.Y.Z` etiketleri kullanılır, `main` dalı kullanılmaz. Senkron, bir güvenlik düzeltmesi ya da ihtiyaç duyulan bir özellik çıktığında yapılır.
  - *Neden:* Etiketli sürümler test edilmiştir.
- **K-03 · Önce config, sonra kod.**
  - *Neden:* Config değişikliği senkronda çakışma üretmez.
- **K-04 · Kodun yeri belli.** Rust kodu `codex-rs/makas/` altında, `codex-makas-*` adlı crate'lerde durur. Diğer her şey kökteki `makas/` altındadır. Upstream dosyalarında sadece tek satırlık kancalar bulunur.
  - *Neden:* Çakışma yüzeyi birkaç satıra iner.
- **K-05 · Dokunuşlar işaretlenir.** Upstream dosyasındaki her değişikliğin üstünde `MAKAS: neden (K-xx)` yorumu bulunur.
  - *Neden:* Neyin bize ait olduğu tek bakışta görülür.
- **K-06 · Silmek yerine tek noktadan kapat.**
  - *Neden:* Silinen kod her senkronda büyük çakışma yaratır.
- **K-11 · Senkron prosedürü.** Senkron `upstream-sync` skill'i ile yapılır: `sync/<etiket>` dalı, merge (rebase değil), check, testler, egress testi, rapor.
  - *Neden:* Süreç tekrarlanabilir olur ve agent'a yaptırılabilir.
- **K-15 · Kalıcı farkları azalt.** Mümkünse genel düzeltmeler önce upstream'e gönderilir.

## Güvenlik ve kilit
- **K-07 · LLM adresi derleme anında sabitlenir.** Adres `MAKAS_LLM_BASE_URL` ile derleme sırasında `codex-makas-config` crate'ine gömülür. Çalışma anında değiştirilemez. Release derlemesinde `MAKAS_LLM_BASE_URL` zorunludur; dev derlemesinde varsayılan `https://aigateway.mycompany.com`'dur.
- **K-08 · Egress testi zorunludur.** Codex süreci izin listesi dışındaki bir adrese bağlanırsa test kırmızı yanar. Bu test zayıflatılamaz; değiştirmek bilgi güvenliğinin onayını gerektirir.
- **K-17 · Kilit çekirdekte.** LLM kilidi ve dış çağrıların kapatılması `codex-core` seviyesinde yapılır. Böylece CLI, app-server, exec ve ileride ACP adaptörü kilidi devralır.
  - *Not:* Hedef kilidi `codex-core`'un altında, `codex-http-client`'ta durur (K-27). Böylece core'dan geçmeyen `login`, `update` ve `cloud` komutları da kapsanır.
- **K-21 · Kişisel kimlik.** Gateway, OpenAI API anahtarları gibi uzun süre kullanılan API anahtarlarıyla çalışır. SSO kullanılmaz. CI botları için ayrı servis anahtarları olur.
- **K-22 · Gateway kalıcı bir bileşendir.** Tek giriş noktasıdır; hem Responses hem Chat Completions sunar. Arkasında vLLM çalışır ve vLLM her iki API'yi de sunar.
- **K-24 · Policy as code.** MCP allowlist ve şablonlar git'te tutulur; MDM oradan dağıtır.
- **K-27 · İki katmanlı kilit.** Birinci katman sağlayıcı kilididir: `codex-core` config'i çözdükten sonra tek sağlayıcı `makas` kalır (`codex-makas-lock`). İkinci katman hedef kilididir: `codex-http-client` ve WebSocket istemcisi, derleme sabitindeki adresin scheme, host ve port'u dışına istek açmaz (`codex-makas-config`). Hedef kontrolü yapamayan SDK'lar kilitli derlemede kapalıdır.
  - *Neden:* Sağlayıcı kilidi config yollarını kapatır, hedef kilidi ise config'den geçmeyen çağrıları (login, update, telemetri, ChatGPT backend) yakalar. Biri atlanırsa diğeri devreye girer. Upstream'in `[application.network]` requirement'ı aynı işi görür ama makinedeki bir dosyaya bağlıdır; bu yüzden kilit koda gömülür.
  - *Sınır:* `codex-http-client` kullanmayan yollar (`otel` blocking istemcisi, `responses-api-proxy`, `--remote`) ve agent'ın çalıştırdığı komutlar bu kilidin dışındadır. İlk üçü kapatma adımında ele alınır.

## Test ve kalite
- **K-09 · Protokol sözleşme testi.** Desktop'ın kullandığı app-server metotları `PROTOKOL.md`'de listelenir; bu metotlarda kırıcı bir değişiklik olursa CI kırmızı yanar.
- **K-10 · Her Makas değişikliğinin testi vardır.** Upstream testleri silinmez.
- **K-28 · Upstream testleri kilit kapalı derlemede çalışır.** Her derleme varsayılan olarak kilitlidir. Upstream testleri için kilit `MAKAS_UNLOCK_FOR_UPSTREAM_TESTS` ile derleme anında kapatılır (`makas/scripts/test-upstream.sh`). `build.rs` bu değişkeni release derlemesinde reddeder. Çalışma anında kilidi açan bir anahtar yoktur.
  - *Neden:* Upstream testleri sahte sunuculara bağlanır ve `openai` sağlayıcısını bekler. Testleri değiştirmek her senkronda çakışma üretir.

## Arayüz (Electron)
- **K-18 · Ara katman.** Electron, app-server'a kendi ara katmanı üzerinden bağlanır (oturum, tur, mesaj, onay, diff). Kavramlar ACP'ye yakın tasarlanır.
- **K-19 · Bağlantı soyutlaması.** Stdio, WebSocket ve WSL/container içinde çalıştırma seçenekleri baştan desteklenir.
- **K-25 · ACP sonraya kalır.** Gerekirse codex-acp adaptörü fork ile derlenip kilitlenerek eklenir.
- **K-26 · T3 Code temel değil, referanstır.** Arayüz sıfırdan yazılır; T3 Code mimari fikirler için incelenir.

## Ad, dil, sürüm
- **K-12 · Commit öneki** `makas(<alan>):` biçimindedir.
- **K-13 · Dil.** Kod, yorum ve commit İngilizce; dokümanlar Türkçe yazılır. *(K-29 ile değiştirildi: commit mesajları artık Türkçe.)*
- **K-14 · Ürün adı Makas.** Kullanıcının gördüğü her yerde Makas (komut, uygulama, installer, `~/.makas`), görmediği her yerde Codex adı kalır (crate'ler, ortam değişkenleri, `.codex/`). LICENSE ve NOTICE korunur, "Codex tabanlı" ibaresi bulunur.
- **K-16 · Sürüm adı** `<upstream>-makas.N` biçimindedir (örnek: `0.164.0-makas.1`).
- **K-20 · Veri dizini** `~/.makas`'tır. Faz 0'da vanilla Codex'e de `CODEX_HOME` ile bu dizin verilir.
- **K-29 · Commit dili Türkçe.** Kod ve yorumlar İngilizce kalır; commit mesajları ve dokümanlar Türkçe yazılır.
  - *Neden:* Proje sahibinin isteği: commit mesajları Türkçe olsun. Kod ve yorumlar İngilizce kalır.
  - *Değiştirdiği karar:* K-13'ün commit kısmı.

## Giriş akışı
- **K-30 · Giriş akışı: API key ve model.** Makas'ta ilk ekran gateway API key'idir; ChatGPT ve Device Code seçenekleri gösterilmez. Key girilince gateway'den model listesi çekilir, kullanıcı modelini seçer ve `config.toml`'a `model` yazılır. Key ya da model yoksa onboarding açılır. Trust adımı bu akışın dışında kalır.
  - *Neden:* ChatGPT girişi kilitli derlemede zaten engelli (K-27); ekranın varsayılan seçimi kullanıcıyı çalışmayan bir yola götürüyordu. Makas sağlayıcısında model listesi çekilmediği için (`api_key_discovery_disabled`) model seçilmezse OpenAI'nin gömülü listesinden bir model seçilirdi.
  - *Nasıl:* Yalnızca kilitli derlemede çalışır (`codex_makas_config::LOCKED`); kilit kapalı upstream testleri aynı kalır. Key doğrulaması ve model listesi `codex-makas-models` üzerinden, yalnızca `llm_base_url()` adresine, `codex-http-client` ile gider (K-27). Gateway anahtarı reddederse ya da ulaşılamazsa key adımı açık kalır. Liste okunamazsa model serbest metinle girilir. Key mevcut API key akışıyla `auth.json`'a yazılır (K-21).
  - *Sınır:* Model-only kurtarma yolunda (key kayıtlı, model yok) TUI anahtarı okumaz; liste yerine serbest metin istenir. Anahtar app-server sınırında kalır.
  - *Not:* Gateway'in `/models` yanıtının OpenAI biçiminde (`data[].id`) olduğu varsayılır (K-22, vLLM). Gerçek gateway ile elle doğrulanmalı.
