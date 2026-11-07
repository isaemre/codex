# Makas: app-server protokolü

> **Kaynak:** upstream `rust-v0.167.0` (`git describe --tags --abbrev=0 --match "rust-v*"` çıktısı). Etiketin commit'i `086ccc833b`, tarihi 2026-11-05.
> **Dosya:** `codex-rs/app-server-protocol/src/protocol/common.rs`. Üretim sırasında dosya etiketle birebir aynıydı.
> **Üretim:** 2026-11-07. Sayılar: istek 183 · sunucu bildirimi 88 · sunucu isteği 11 · istemci bildirimi 1.

Bu dosya, Makas masaüstü arayüzünün (`makas/desktop/`, Faz 2) kullanacağı app-server metotlarının envanteridir (K-09). Bu metotlarda kırıcı bir değişiklik olursa CI kırmızı yanmalıdır; bu kontrol henüz yazılmadı.

## Nasıl okunur

- Makas arayüzü, Electron ara katmanı üzerinden bir `codex app-server` sürecine JSON-RPC 2.0 ile bağlanır. Üç yön vardır:
  - **İstemci → sunucu istekleri:** istemci gönderir, sunucu `result` veya `error` ile yanıtlar.
  - **Sunucu → istemci bildirimleri:** sunucu yanıt beklemeden gönderir; veri `params` içindedir.
  - **Sunucu → istemci istekleri:** sunucu yanıt bekler. Onaylar ve kullanıcı girdisi bu gruptadır.
- Kavramlar: **thread** bir oturum, **tur (turn)** bir kullanıcı isteği ve ona bağlı iştir, **öğe (item)** bir turun parçasıdır (mesaj, komut, dosya değişikliği vb.).
- Her grupta **Çekirdek** tablosu, Makas masaüstünün temel ekranlarını doğrudan besleyen metotları içerir: oturum listesi, mesajlaşma, akan yanıt, komut çıktısı, diff ve onaylar. **Diğer** geri kalanı toplar.
- Metot adları upstream'deki gibidir. Açıklamalar ölçülüdür: doc yorumlarına, `codex-rs/app-server/README.md` dosyasına, resmi dokümanın bir kopyasına ve alan adlarına dayanır. Bu kaynakların hiçbirinde açıklaması olmayan metotların açıklamaları yalnızca ad ve alanlardan çıkarıldı.
- Deneysel kapı: `initialize` içinde `capabilities.experimentalApi: true` gönderilmezse deneysel bir istek `<metot> requires experimentalApi capability` hatasıyla reddedilir; deneysel bir bildirim bağlantıya hiç gönderilmez. `initialize` öncesi her istek `Not initialized` hatası alır.

### Durum etiketleri

| Etiket | Anlamı |
|---|---|
| `deneysel` | Metot `#[experimental]` ile işaretli. Yukarıdaki kapı uygulanır. |
| `kısmen deneysel` | Metot kararlı; yalnızca bazı istek alanları deneysel. Alanlar Ek bölümünde. |
| `deneysel (yorumda)` | Doc yorumunda "EXPERIMENTAL" yazıyor, ama makroyla kapılı değil. Sunucu bunu experimentalApi kapısı olmadan kabul eder ya da gönderir. |
| `kararsız` | Kodda `[UNSTABLE]` olarak işaretli; biçimi yakında değişebilir. |
| `geliştirme aşamasında` | Depodaki README metodu geliştirme aşamasında işaretler ve üretim istemcilerinden henüz çağrılmamasını ister. |
| `kullanımdan kalkmış` | Kodda `DEPRECATED` olarak işaretli. |
| `dahili` | Yalnızca dahili kullanım için işaretli. |
| `test` | Yalnızca test için. |

## 1. İstemci → sunucu istekleri (183)

Her istek bir `result` veya `error` ile yanıtlanır.

### Çekirdek

| Metot | Ekran | Ne için | Durum |
|---|---|---|---|
| `initialize` | Bağlantı | Bağlantıyı başlatır. İstemci bilgisi ve yetenekleri (ör. `experimentalApi`, `optOutNotificationMethods`) bildirilir; yanıtta kullanıcı aracısı, `codexHome` ve platform bilgisi döner. |  |
| `thread/list` | Oturum listesi | Kayıtlı thread'leri sayfalı ve filtreli listeler (çalışma durumu dahil). Hariç tutma listesi, sıralama ve filtre alır. | kısmen deneysel |
| `thread/loaded/list` | Oturum listesi | Bellekte yüklü thread kimliklerini sayfalı listeler. |  |
| `thread/start` | Oturum listesi | Yeni bir thread (oturum) oluşturur; thread'in tur ve öğe olaylarına otomatik abone olur. | kısmen deneysel |
| `thread/resume` | Oturum listesi | Kayıtlı bir thread'i kimliğiyle yeniden açar; sonraki `turn/start` çağrıları bu thread'e ekler. | kısmen deneysel |
| `thread/fork` | Oturum listesi | Thread geçmişini kopyalayarak yeni bir thread açar. `lastTurnId` ile kesilebilir, `ephemeral` ile bellek içi olabilir. | kısmen deneysel |
| `thread/read` | Oturum listesi | Kayıtlı bir thread'in meta verisini, thread'i yüklemeden okur. `includeTurns` ile tur bilgisi de istenebilir. |  |
| `thread/name/set` | Oturum listesi | Yüklü veya kayıtlı bir thread'in adını ayarlar. `thread/name/updated` yayınlar. |  |
| `thread/archive` | Oturum listesi | Thread kaydını arşiv dizinine taşır; kayıtlı alt thread'ler de arşivlenir. `thread/archived` yayınlar. |  |
| `thread/unarchive` | Oturum listesi | Arşivlenmiş thread'i etkin oturumlara geri getirir. `thread/unarchived` yayınlar. |  |
| `thread/delete` | Oturum listesi | Kalıcı thread'i ve oluşturduğu alt thread'leri kalıcı olarak siler. `thread/deleted` yayınlar. |  |
| `thread/unsubscribe` | Oturum listesi | Bu bağlantının thread olaylarından aboneliğini kaldırır. Son abone ayrılırsa thread bir süre sonra kapanır. |  |
| `turn/start` | Mesajlaşma | Thread'e kullanıcı girdisi (veya bağımsız araç çıktısı) ekler ve üretimi başlatır. İlk `turn` yanıtı döner, olaylar akar. | kısmen deneysel |
| `turn/steer` | Mesajlaşma | Aktif tura ek kullanıcı girdisi ekler; yeni tur açmaz. | kısmen deneysel |
| `turn/interrupt` | Mesajlaşma | Devam eden turu iptal etmeyi ister. Tur `interrupted` durumuyla biter. |  |
| `thread/turns/list` | Mesajlaşma | Kayıtlı bir thread'in tur geçmişini sayfalı okur; thread'i yüklemez. |  |
| `thread/items/list` | Mesajlaşma | Kalıcı thread öğelerini sayfalı listeler; isteğe bağlı olarak tek bir tura göre filtreler. |  |
| `command/exec` | Komut çıktısı | Thread veya tur açmadan tek bir komutu (argv) sunucunun sandbox'ı altında çalıştırır. Çıktı `command/exec/outputDelta` ile akar. | kısmen deneysel |
| `command/exec/write` | Komut çıktısı | Çalışan bir `command/exec` oturumuna stdin baytı yazar veya stdin'i kapatır. |  |
| `command/exec/resize` | Komut çıktısı | PTY tabanlı bir `command/exec` oturumunun boyutunu değiştirir. |  |
| `command/exec/terminate` | Komut çıktısı | Çalışan bir `command/exec` oturumunu sonlandırır. |  |
| `initialized` | Bağlantı | İstemci → sunucu bildirimi. `initialize` isteğinden sonra gönderilir; yanıt beklemez. |  |

- **Diff:** Bu envanterde diff için ayrı bir istek yok. Diff verisi `turn/diff/updated` ve `item/fileChange/patchUpdated` bildirimleriyle ve `fileChange` öğeleriyle gelir (bkz. 2. grup).
- **Onaylar:** Onaylar istemciden değil sunucudan gelir (bkz. 3. grup).

### Diğer

#### Thread yönetimi (30)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/sample` | Bir thread'in beklenen turundan örnek bir model yanıtı üretir. Bekleme, ana thread'deki istekleri engellemez. | deneysel |
| `thread/sample/cancel` | Devam eden thread örneklemesini iptal eder. | deneysel |
| `thread/increment_elicitation` | Thread'in dış elicitation sayacını artırır; bekleyen bir onay sırasında zaman aşımı sayımını durdurur. | deneysel |
| `thread/decrement_elicitation` | Dış elicitation sayacını azaltır; sayaç sıfıra inince zaman aşımı sayımı yeniden başlar. | deneysel |
| `thread/prediction/request` | Kaynak tur (`sourceTurnId`) için bir thread tahmini (prediction) ister. | deneysel |
| `thread/goal/set` | Thread hedefini (goal) ayarlar. `thread/goal/updated` yayınlar. |  |
| `thread/goal/get` | Thread'in mevcut hedefini okur. |  |
| `thread/goal/clear` | Thread hedefini temizler. `thread/goal/cleared` yayınlar. |  |
| `thread/metadata/update` | Thread meta verisini (Git bilgisi, proje, Daybreak tercihi) günceller. | kısmen deneysel |
| `thread/attachment/add` | Kalıcı bir kaynak referansını thread'e ek (attachment) olarak ekler. Aynı tür ve kimlik tekrar yazılırsa mevcut ek döner. |  |
| `thread/attachment/list` | Bir thread'in eklerini sayfalı listeler. |  |
| `thread/attachmentOwner/list` | Ek kimliğine göre ters arama yapar: bu eki hangi thread'lerin taşıdığını listeler. |  |
| `thread/attachment/remove` | Bir eki kararlı, thread'e özgü kimliğiyle siler. |  |
| `thread/section/move` | Thread'i bir bölüme taşır, bölüm içinde sıralar veya bölümden çıkarır (`sectionId: null`). |  |
| `thread/settings/update` | Thread ayarlarını günceller (ör. devre dışı plugin'ler, `cwd`, onay ilkesi). | deneysel |
| `thread/compact/start` | Thread geçmişinin sıkıştırılmasını (compaction) başlatır; ilerleme `turn/*` ve `item/*` bildirimleriyle akar. |  |
| `thread/shellCommand` | Thread bağlamında kullanıcı başlatmalı bir kabuk komutu çalıştırır. Komut sandbox dışında ve tam erişimle çalışır. |  |
| `thread/approveGuardianDeniedAction` | Guardian'ın reddettiği bir eylemi onaylar. `event` alanı serileştirilmiş bir GuardianAssessmentEvent'tir. |  |
| `thread/backgroundTerminals/clean` | Thread'in çalışan tüm arka plan terminallerini durdurur. | deneysel |
| `thread/backgroundTerminals/list` | Yüklü bir thread'in çalışan arka plan terminallerini listeler. | deneysel |
| `thread/backgroundTerminals/terminate` | Arka plan terminallerinden birini `processId` ile sonlandırır. | deneysel |
| `thread/revert` | Sayfalı bir thread'in kalıcı geçmişini, bir turdan önceki önekle değiştirir; o tur ve sonrakiler çıkarılır. `thread/reverted` yayınlar. |  |
| `thread/search` | Thread'lerde metin araması yapar (`searchTerm` zorunlu). Sayfalı çalışır; sıralama, kaynak ve arşiv filtresi alır. | deneysel |
| `thread/searchOccurrences` | Bir thread içindeki görünür mesajlarda büyük/küçük harf duyarsız metin geçişlerini arar (sayfalı). | deneysel |
| `thread/readState/update` | Thread'i okundu veya okunmadı olarak işaretler. `expectedRevision` eşleşmezse çakışma döner. | deneysel |
| `thread/items/read` | Bir turdan kalıcı öğelerin sınırlı bir kümesini (1–100 kimlik) getirir. | deneysel |
| `thread/guardianReview` | Boşta bekleyen thread'in son enjekte edilmiş `exec_command`'ını çalıştırmadan inceletir; bağlantıyı thread'e abone eder. | deneysel |
| `thread/inject_items` | Yüklü bir thread'in modele görünen geçmişine ham Responses öğeleri ekler; kullanıcı turu başlatmaz. |  |
| `turn/settings/update` | Çalışan bir turun ayarlarını (ör. model, efor, onaylayıcı) değiştirir; sonraki turları etkilemez. | deneysel |
| `review/start` | Thread üzerinde reviewer'ı çalıştırır. `enteredReviewMode` ve `exitedReviewMode` öğeleri üretir. |  |

#### Proje ve bölümler (11)

| Metot | Ne için | Durum |
|---|---|---|
| `project/list` | Proje kayıtlarını sayfalı listeler. | deneysel |
| `project/read` | Bir proje kaydını okur. | deneysel |
| `project/create` | Yeni bir proje kaydı oluşturur. | deneysel |
| `project/import` | Bir projeyi ve thread'lerini içe aktarır. | deneysel |
| `project/update` | Proje adını, köklerini veya meta verisini günceller. | deneysel |
| `project/move` | Bir projeyi listede başka bir projenin önüne taşır. | deneysel |
| `project/delete` | Proje kaydını siler. | deneysel |
| `threadSection/list` | Bağımsız kalıcı thread bölümlerini listeler. |  |
| `threadSection/create` | Yeni bir thread bölümü oluşturur. |  |
| `threadSection/update` | Bir thread bölümünün adını veya görünümünü günceller. |  |
| `threadSection/delete` | Bir thread bölümünü siler. |  |

#### Thread kuyruğu (6)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/queue/add` | Thread'in gönderim kuyruğuna bir girdi ekler. | deneysel |
| `thread/queue/list` | Thread'in gönderim kuyruğunu sayfalı listeler. | deneysel |
| `thread/queue/update` | Kuyruktaki bir gönderimi günceller. | deneysel |
| `thread/queue/delete` | Kuyruktaki bir gönderimi siler. | deneysel |
| `thread/queue/reorder` | Kuyruktaki gönderimlerin sırasını değiştirir. | deneysel |
| `thread/queue/start` | Kuyruktaki bir gönderimi tur olarak başlatır. | deneysel |

#### Bellek ve depolama (5)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/memoryMode/set` | Thread'in bellek modunu ayarlar. | deneysel |
| `thread/memory/read` | Yüklü bir thread'in bellek özetini okur. Bellek kapalıysa veya özet yoksa `null` döner. | deneysel |
| `memory/status` | Bellek konsolidasyonunun durumunu okur. | deneysel |
| `memory/reset` | Bellek durumunu sıfırlar. | deneysel |
| `rollout/compress` | Soğuk yerel kayıtlar (rollout) için arka planda en iyi çaba sıkıştırma başlatır. | deneysel |

#### Gerçek zamanlı (7)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/realtime/start` | Thread'e bağlı bir gerçek zamanlı oturum başlatır. | deneysel |
| `thread/realtime/appendAudio` | Gerçek zamanlı oturuma ses girdisi ekler. | deneysel |
| `thread/realtime/appendText` | Gerçek zamanlı oturuma metin girdisi ekler. | deneysel |
| `thread/realtime/appendSpeech` | Gerçek zamanlı oturuma seslendirilebilir metin ekler. | deneysel |
| `thread/realtime/stop` | Gerçek zamanlı oturumu durdurur. | deneysel |
| `thread/timeline/list` | Normal ve gerçek zamanlı thread geçmişini kayıt sırasıyla sayfalı listeler. | deneysel |
| `thread/realtime/listVoices` | Gerçek zamanlı oturumda desteklenen sesleri listeler. | deneysel |

#### Model ve özellikler (7)

| Metot | Ne için | Durum |
|---|---|---|
| `model/list` | Kullanılabilir modelleri efor seçenekleri ve girdi modaliteleriyle listeler (`includeHidden` ile gizliler dahil). |  |
| `modelProvider/capabilities/read` | Model ve sağlayıcı kombinasyonları için sağlayıcı yetenek sınırlarını okur. |  |
| `experimentalFeature/list` | Özellik bayraklarını yaşam döngüsü aşaması bilgisiyle sayfalı listeler. |  |
| `permissionProfile/list` | Beta izin profillerini ve etkin gereksinimlerce izin verilip verilmediğini listeler. |  |
| `experimentalFeature/enablement/set` | Desteklenen özellik anahtarları (ör. `apps`, `plugins`) için çalışma zamanı ayarını bellekte değiştirir. |  |
| `collaborationMode/list` | İşbirliği modu ön ayarlarını listeler (sayfalama yok). | deneysel |
| `mock/experimentalMethod` | Yalnızca test: deneysel kapının çalıştığını doğrulamak için. | deneysel (test) |

#### Hesap ve kimlik (23)

| Metot | Ne için | Durum |
|---|---|---|
| `userVerification/status` | Yerel biyometrik doğrulamanın durumunu okur: kimlik bilgisi kimliği ve kullanılamama nedeni. | deneysel |
| `userVerification/enroll` | Yerel doğrulama için kimlik bilgisi kaydeder; yanıtta kimlik ve genel anahtar döner. | deneysel |
| `userVerification/delete` | Kayıtlı yerel doğrulama kimlik bilgisini siler. | deneysel |
| `userVerification/verify` | Yerel imza anahtarıyla doğrulama kanıtı üretir; bekleyen bir elicitation'a bağlı değildir. | deneysel |
| `userVerification/cancel` | Aynı bağlantıdaki bir yerel doğrulama RPC'sini (`requestId`) iptal eder; yanıt boş bir onaydır. | deneysel |
| `account/gatewayOAuth/read` | Gateway OAuth girişinin durumunu okur. |  |
| `account/gatewayOAuth/login` | Gateway OAuth girişini başlatır. |  |
| `account/gatewayOAuth/cancel` | Devam eden gateway OAuth girişini iptal eder. |  |
| `account/login/start` | Hesap girişi başlatır. Yöntem (API anahtarı, ChatGPT tarayıcı veya cihaz kodu) parametreyle seçilir. | kısmen deneysel |
| `account/bedrock/discover` | Amazon Bedrock kurulumu için kaynakları keşfeder. | deneysel |
| `account/bedrock/setup` | Amazon Bedrock sağlayıcısını kurar. | deneysel |
| `account/gcp/setup` | Yerleşik Vertex AI sağlayıcısını yapılandırır. | deneysel |
| `account/bedrock/checkGovCloudRequirements` | Bedrock GovCloud gereksinimlerini denetler; yapılandırmayı yeniden yükler. | deneysel |
| `account/login/cancel` | Devam eden hesap girişini iptal eder. |  |
| `accountSession/list` | Yönetilen ChatGPT girişinin oturumlarını listeler. | deneysel |
| `accountSession/switch` | Bir oturum kimliği ve hedef çalışma alanı ile oturum değiştirir. | deneysel |
| `account/logout` | Hesabın oturumunu kapatır. |  |
| `account/rateLimits/read` | Hesabın hız sınırı bilgisini okur. |  |
| `account/rateLimitResetCredit/consume` | Kazanılmış bir hız sınırı sıfırlama hakkını tüketir (çağıranın verdiği `idempotencyKey` ile). |  |
| `account/usage/read` | Token kullanım bilgisini okur. |  |
| `account/workspaceMessages/read` | Etkin çalışma alanı mesajlarını okur. |  |
| `account/sendAddCreditsNudgeEmail` | Çalışma alanı sahibine kredi veya kullanım sınırı hakkında e-posta gönderilmesini ister. |  |
| `account/read` | Mevcut hesap durumunu okur. |  |

#### Yapılandırma ve gereksinimler (10)

| Metot | Ne için | Durum |
|---|---|---|
| `windowsSandbox/setupStart` | Windows sandbox kurulumunu başlatır. Bitince `windowsSandbox/setupCompleted` gelir. |  |
| `windowsSandbox/readiness` | Windows sandbox'ının hazır olup olmadığını denetler. `updateRequired` dönerse `setupStart` çağrılır. |  |
| `config/read` | Yapılandırma katmanları birleştirildikten sonra etkin yapılandırmayı diskten okur. |  |
| `externalAgentConfig/detect` | Taşınabilir dış agent yapılandırma öğelerini (ana dizin ve isteğe bağlı `cwd`) tespit eder. |  |
| `externalAgentConfig/import` | Seçilen dış agent yapılandırma öğelerini içe aktarır. İlerleme ve tamamlanma bildirimleri gelir. |  |
| `externalAgentConfig/import/recordHistory` | Bir içe aktarma geçmişine sonuç kaydeder. |  |
| `externalAgentConfig/import/readHistories` | İçe aktarma geçmişlerini okur. |  |
| `config/value/write` | Kullanıcının `config.toml` dosyasında tek bir anahtarı ve değerini yazar. |  |
| `config/batchWrite` | Yapılandırma değişikliklerini `config.toml` üzerinde toplu ve atomik olarak uygular. |  |
| `configRequirements/read` | `requirements.toml` ve/veya MDM gereksinimlerini okur (izin listeleri, sabitlenmiş `featureRequirements`). |  |

#### Skill, hook, pazar yeri, eklenti ve uygulama (23)

| Metot | Ne için | Durum |
|---|---|---|
| `skills/list` | Bir veya daha fazla `cwd` için skill'leri listeler (`forceReload` ile yeniden yükleme yapılabilir). |  |
| `skills/extraRoots/set` | Skill keşfi için ek kökleri süreç düzeyinde değiştirir; kalıcı olarak kaydetmez. |  |
| `hooks/list` | Bulunan yaşam döngüsü hook'larını bir veya daha fazla `cwd` için listeler. |  |
| `marketplace/add` | Uzak bir plugin pazar yerini ekler ve kullanıcı yapılandırmasına kaydeder. |  |
| `marketplace/remove` | Bir pazar yerini ve kurulu kökünü kaldırır. |  |
| `marketplace/upgrade` | Git tabanlı pazar yerlerini yeniler; ad verilmezse hepsini. |  |
| `plugin/list` | Pazar yerlerini ve eklenti durumunu listeler. | geliştirme aşamasında |
| `plugin/search` | Eklenti arar; sonuçlar keşif davranışını korur. | deneysel |
| `plugin/installed` | Kurulu eklentileri listeler. |  |
| `plugin/reconcile` | Eklenti paketi ve kurulu durumdaki değişiklikleri tarar ve uzlaştırır. Çalışma hazırlığı onayı değildir. |  |
| `plugin/read` | Bir eklentiyi okur: paketlenmiş skill'ler, uygulamalar ve MCP sunucu adları. | geliştirme aşamasında |
| `plugin/skill/read` | Uzak bir eklentinin skill'inin Markdown içeriğini isteğe bağlı okur. |  |
| `plugin/share/save` | Eklentiyi paylaşılabilir olarak uzak kataloğa kaydeder; keşfedilebilirliği ve paylaşım hedeflerini ayarlar. |  |
| `plugin/share/updateTargets` | Paylaşılan bir eklentinin keşfedilebilirliğini ve paylaşım hedeflerini günceller. |  |
| `plugin/share/list` | Paylaşılan eklentileri listeler. |  |
| `plugin/share/checkout` | Paylaşılan uzak bir eklentiyi yerel bir yola açar. |  |
| `plugin/share/delete` | Paylaşılan uzak bir eklentiyi siler. |  |
| `app/read` | Belirli uygulama (connector) kimlikleri için meta veriyi ve isteğe bağlı görüntüleme amaçlı araç özetini okur. Sunucu bunu experimentalApi kapısı olmadan kabul eder. | deneysel (yorumda) |
| `app/list` | Kullanılabilir uygulamaları (connector) erişilebilirlik ve etkinlik meta verisiyle sayfalı listeler. Sunucu bunu experimentalApi kapısı olmadan kabul eder. | deneysel (yorumda) |
| `app/installed` | Kurulu uygulamaların çalışma zamanı durumunu okur (etkin ve çağrılabilir durumlar). |  |
| `skills/config/write` | Bir skill'i yoluna göre etkinleştirir veya devre dışı bırakır. |  |
| `plugin/install` | Bir eklentiyi pazar yerinden kurar. |  |
| `plugin/uninstall` | Kurulu bir eklentiyi kaldırır. |  |

#### MCP (7)

| Metot | Ne için | Durum |
|---|---|---|
| `mcpServer/oauth/login` | Yapılandırılmış bir MCP sunucusu için OAuth girişi başlatır. Dönen URL açılır, `loginId` ile tamamlanma izlenir. |  |
| `config/mcpServer/reload` | MCP sunucu yapılandırmasını diskten yeniden yükler; yüklü thread'ler için yenileme kuyruğa alınır. |  |
| `mcpServerStatus/list` | MCP sunucularını, araçlarını, kaynaklarını ve kimlik durumunu sayfalı listeler. |  |
| `mcpServer/resource/read` | Başlatılmış bir MCP sunucusundan tek bir kaynağı okur. |  |
| `mcpServer/event/stream/start` | Bir MCP sunucusu için olay akışı aboneliği başlatır. | deneysel |
| `mcpServer/event/stream/stop` | Bir MCP olay akışı aboneliğini durdurur. | deneysel |
| `mcpServer/tool/call` | Bir thread'in yapılandırılmış MCP sunucusunda araç çağırır. |  |

#### Dosya sistemi (9)

| Metot | Ne için | Durum |
|---|---|---|
| `fs/readFile` | Ana makinedeki bir dosyayı okur. |  |
| `fs/writeFile` | Ana makinedeki bir dosyaya yazar. |  |
| `fs/createDirectory` | Ana makinede dizin oluşturur. |  |
| `fs/getMetadata` | Mutlak bir yolun meta verisini okur. |  |
| `fs/readDirectory` | Bir dizinin doğrudan alt öğe adlarını listeler. |  |
| `fs/remove` | Bir dosyayı veya dizin ağacını siler. |  |
| `fs/copy` | Bir dosyayı veya dizin ağacını kopyalar. |  |
| `fs/watch` | Mutlak bir yol için dosya sistemi izleme bildirimleri başlatır (`fs/changed`). |  |
| `fs/unwatch` | Önceki bir `fs/watch` izlemesini durdurur. |  |

#### Süreç (4)

| Metot | Ne için | Durum |
|---|---|---|
| `process/spawn` | Codex sandbox'ı olmadan bağımsız bir süreç (argv) başlatır. | deneysel |
| `process/writeStdin` | Çalışan bir `process/spawn` sürecine stdin yazar veya stdin'i kapatır. | deneysel |
| `process/kill` | Çalışan bir `process/spawn` sürecini sonlandırır. | deneysel |
| `process/resizePty` | PTY tabanlı bir `process/spawn` sürecinin boyutunu değiştirir. | deneysel |

#### Dosya arama (4)

| Metot | Ne için | Durum |
|---|---|---|
| `fuzzyFileSearch` | Eski tek seferlik dosya arama. `cancellationToken` ile önceki bir sorguyu iptal eder. |  |
| `fuzzyFileSearch/sessionStart` | Dosya arama oturumu başlatır. | deneysel |
| `fuzzyFileSearch/sessionUpdate` | Dosya arama oturumundaki sorguyu günceller. | deneysel |
| `fuzzyFileSearch/sessionStop` | Dosya arama oturumunu durdurur. | deneysel |

#### Uzaktan kontrol ve ortam (10)

| Metot | Ne için | Durum |
|---|---|---|
| `remoteControl/enable` | Uzaktan kontrolü etkinleştirir. | deneysel |
| `remoteControl/disable` | Uzaktan kontrolü devre dışı bırakır. | deneysel |
| `remoteControl/status/read` | Uzaktan kontrol bağlantısının durumunu okur. | deneysel |
| `remoteControl/pairing/start` | Uzaktan kontrol eşleştirmesi başlatır. | deneysel |
| `remoteControl/pairing/status` | Uzaktan kontrol eşleştirme durumunu okur. | deneysel |
| `remoteControl/client/list` | Uzaktan kontrol istemcilerini listeler. | deneysel |
| `remoteControl/client/revoke` | Bir uzaktan kontrol istemcisinin erişimini iptal eder. | deneysel |
| `environment/add` | Uzak bir ortamı kimliğiyle ekler veya değiştirir; sonra seçilebilir. | deneysel |
| `environment/info` | Yapılandırılmış bir ortama bağlanır; kabuk ve varsayılan `cwd` bilgisini okur. | deneysel |
| `environment/status` | Yapılandırılmış bir ortamın güncel durumunu okur. | deneysel |

#### Geri bildirim ve teşhis (3)

| Metot | Ne için | Durum |
|---|---|---|
| `server/diagnostics` | İçerik içermeyen, yalnızca süreç yerel tanılama bilgisi (süreç ve ölçüm değerleri) döner. | deneysel |
| `feedback/upload` | Geri bildirim raporu gönderir: sınıflandırma, isteğe bağlı neden ve loglar. |  |
| `feedback/save` | Geri bildirim raporunu verilen yola ZIP dosyası olarak kaydeder. |  |

#### Kullanımdan kalkmış (v1) (3)

| Metot | Ne için | Durum |
|---|---|---|
| `getConversationSummary` | Rollout yolu veya thread kimliğiyle konuşma özetini okur. | kullanımdan kalkmış |
| `gitDiffToRemote` | Çalışma dizininin uzak tabanına göre git diff'ini döner. | kullanımdan kalkmış |
| `getAuthStatus` | Eski kimlik durumu. Yerine `account/read` kullanılır. | kullanımdan kalkmış |

## 2. Sunucu → istemci bildirimleri (88)

Sunucu bunları yanıt beklemeden gönderir; veri `params` içindedir.

### Çekirdek

| Metot | Ekran | Ne için | Durum |
|---|---|---|---|
| `thread/started` | Oturum listesi | Yeni bir thread oluşturulduğunda yayınlanır (ör. `thread/start`, `thread/fork`). |  |
| `thread/status/changed` | Oturum listesi | Yüklü bir thread'in çalışma durumu (`notLoaded`, `idle`, `systemError`, `active`) değiştiğinde yayınlanır. |  |
| `thread/name/updated` | Oturum listesi | Thread adı değiştiğinde yayınlanır. |  |
| `thread/archived` | Oturum listesi | Thread arşivlendiğinde yayınlanır. |  |
| `thread/unarchived` | Oturum listesi | Thread arşivden çıkarıldığında yayınlanır. |  |
| `thread/deleted` | Oturum listesi | Thread kalıcı olarak silindiğinde yayınlanır. |  |
| `thread/closed` | Oturum listesi | Thread, son abonesi ayrıldıktan sonra bellekten kaldırıldığında yayınlanır. |  |
| `turn/started` | Mesajlaşma | Yeni bir tur başladığında yayınlanır. `items` boştur, durum `inProgress`'tir. |  |
| `turn/completed` | Mesajlaşma | Tur tamamlandığında, kesildiğinde veya başarısız olduğunda yayınlanır. Nihai durumu taşır. |  |
| `item/started` | Mesajlaşma | Yeni bir öğe (mesaj, komut, dosya değişikliği vb.) başladığında tam öğeyi gönderir. |  |
| `item/completed` | Mesajlaşma | Bir öğe bittiğinde nihai hâlini gönderir. Yetkili durum budur. |  |
| `error` | Mesajlaşma | Tur sırasında hata oluştuğunda yayınlanır. `willRetry` true ise sunucu yeniden dener ve tur aktif kalır. |  |
| `item/agentMessage/delta` | Akan yanıt | Agent mesajına akan metin parçası ekler. |  |
| `item/reasoning/summaryTextDelta` | Akan yanıt | Akan düşünme özeti parçası. `summaryIndex` yeni bir özet bölümünü gösterir. |  |
| `item/reasoning/summaryPartAdded` | Akan yanıt | Yeni bir düşünme özeti bölümünün başladığını bildirir. |  |
| `item/reasoning/textDelta` | Akan yanıt | Model destekliyorsa ham düşünme metninin parçası. |  |
| `item/commandExecution/outputDelta` | Komut çıktısı | Bir komut öğesinin stdout/stderr parçası; sırayla eklenmelidir. |  |
| `item/commandExecution/terminalInteraction` | Komut çıktısı | Çalışan bir komuta stdin gönderildiğinde yayınlanır. |  |
| `command/exec/outputDelta` | Komut çıktısı | `command/exec` akışının base64 stdout/stderr parçası. Bağlantıya özeldir; bağlantı kapanırsa süreç sonlandırılır. |  |
| `turn/diff/updated` | Diff | Turun birleşik diff'i değiştiğinde yayınlanır. Turdaki tüm dosya değişikliklerinin toplamıdır. |  |
| `item/fileChange/patchUpdated` | Diff | Bir dosya değişikliği öğesinin yamaları güncellendiğinde yayınlanır. |  |
| `serverRequest/resolved` | Onaylar | Bekleyen bir sunucu isteği (ör. onay) yanıtlandığında veya temizlendiğinde yayınlanır. |  |

### Diğer

#### Thread olayları (15)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/readState/changed` | Thread'in okundu durumu değiştiğinde yayınlanır. | deneysel |
| `thread/reverted` | `thread/revert` sonrasında thread geçmişinin değiştiğini bildirir. |  |
| `thread/section/changed` | Bir thread bölüm değiştirdiğinde veya bölüm içinde sırası değiştiğinde yayınlanır. |  |
| `thread/attachment/updated` | Bir thread'in eklerinde (attachment) değişiklik olduğunda yayınlanır. |  |
| `thread/goal/updated` | Thread hedefi ayarlandığında veya güncellendiğinde yayınlanır. |  |
| `thread/prediction/updated` | Bir thread tahmini (prediction) sonucu güncellendiğinde yayınlanır. | deneysel |
| `thread/goal/cleared` | Thread hedefi temizlendiğinde yayınlanır. |  |
| `thread/queue/changed` | Thread'in gönderim kuyruğu değiştiğinde yayınlanır. | deneysel |
| `project/changed` | Bir proje kaydı eklendiğinde, güncellendiğinde veya silindiğinde yayınlanır. | deneysel |
| `thread/project/updated` | Bir thread'in projesi değiştiğinde yayınlanır. | deneysel |
| `thread/environment/connected` | Bir thread bir ortama bağlandığında yayınlanır. | deneysel |
| `thread/environment/disconnected` | Bir thread'in ortam bağlantısı koptuğunda yayınlanır. | deneysel |
| `thread/settings/updated` | Thread ayarları değiştiğinde yayınlanır. | deneysel |
| `thread/tokenUsage/updated` | Aktif thread'in token kullanımı güncellendiğinde yayınlanır. |  |
| `thread/compacted` | Eski sıkıştırma bildirimi. Yerine `contextCompaction` öğesi kullanılır. | kullanımdan kalkmış |

#### Tur ve öğe olayları (9)

| Metot | Ne için | Durum |
|---|---|---|
| `hook/started` | Senkron bir yaşam döngüsü hook'u başladığında yayınlanır. |  |
| `hook/completed` | Hook'un nihai çalıştırma özeti hazır olduğunda yayınlanır. |  |
| `turn/plan/updated` | Agent planını paylaştığında veya değiştirdiğinde yayınlanır; her adımın durumunu taşır. |  |
| `rawResponseItem/completed` | Yalnızca dahili kullanım (Codex Cloud) için tamamlanmış ham Responses öğesi. | dahili |
| `rawResponse/completed` | Yalnızca dahili kullanım. İstemcinin kesin upstream kullanım verisini almasını sağlar. | dahili |
| `item/plan/delta` | Plan öğesine akan plan metni parçası ekler. Son `plan` öğesi birleşik parçalarla birebir aynı olmayabilir. Sunucu bunu experimentalApi kapısı olmadan gönderir. | deneysel (yorumda) |
| `item/fileChange/outputDelta` | Eski `apply_patch` metin çıktısı. Güncel sürümler göndermez; `fileChange` öğesi ve `turn/diff/updated` kullanılır. | kullanımdan kalkmış |
| `item/mcpToolCall/progress` | MCP araç çağrısının ilerleme mesajını bildirir. |  |
| `turn/moderationMetadata` | Tur için moderasyon meta verisini bildirir. | deneysel |

#### Otomatik onay incelemesi (4)

| Metot | Ne için | Durum |
|---|---|---|
| `item/autoApprovalReview/started` | Otomatik onay incelemesi başladığında yayınlanır. | kararsız |
| `item/autoApprovalReview/completed` | Otomatik onay incelemesi bittiğinde karar kaynağıyla birlikte yayınlanır. | kararsız |
| `autoApprovalReview/strictReviewRequired` | Otomatik onay incelemesinin sıkı insan incelemesi gerektirdiğini bildirir. | deneysel |
| `guardianWarning` | Guardian (otomatik onay) uyarısı. Kullanıcıya gösterilecek kısa mesajı taşır. |  |

#### Model, uyarı ve bildirimler (10)

| Metot | Ne için | Durum |
|---|---|---|
| `model/rerouted` | Hizmet bir isteği başka bir modele yönlendirdiğinde yayınlanır. |  |
| `model/verification` | Hesabın ek doğrulama gerektirdiği durumda yayınlanır. |  |
| `modelProvider/authRecoveryStarted` | Model sağlayıcısının kimlik kurtarma işlemi başladığında yayınlanır. |  |
| `modelProvider/authRecoveryCompleted` | Model sağlayıcısının kimlik kurtarma işlemi bittiğinde yayınlanır. |  |
| `model/safetyBuffering/updated` | Bir yanıt geçici güvenlik tamponlamasına girdiğinde yayınlanır. |  |
| `warning` | Kritik olmayan çalışma zamanı uyarısını bildirir; thread'e bağlı olabilir. |  |
| `deprecationNotice` | Kullanılan bir özelliğin kullanımdan kalkacağını bildirir. |  |
| `configWarning` | Yapılandırma veya başlatma sorununda kurtarılabilir uyarı. |  |
| `windows/worldWritableWarning` | Windows'ta sandbox'ın koruyamadığı, herkese yazılabilir dizinleri bildirir. |  |
| `windowsSandbox/setupCompleted` | `windowsSandbox/setupStart` bittiğinde sonucu bildirir. |  |

#### Hesap (4)

| Metot | Ne için | Durum |
|---|---|---|
| `account/updated` | Hesap durumu değiştiğinde yayınlanır. |  |
| `account/gatewayOAuth/changed` | Gateway OAuth durumu değiştiğinde yayınlanır. |  |
| `account/rateLimits/updated` | Hesabın hız sınırı bilgisi güncellendiğinde yayınlanır. |  |
| `account/login/completed` | Hesap girişi tamamlandığında yayınlanır. |  |

#### MCP, skill ve uygulama (5)

| Metot | Ne için | Durum |
|---|---|---|
| `skills/changed` | İzlenen yerel skill dosyaları değiştiğinde yayınlanır. |  |
| `mcpServer/oauthLogin/completed` | `mcpServer/oauth/login` akışı bittiğinde yayınlanır; `loginId` ile eşleşir. |  |
| `mcpServer/startupStatus/updated` | Yüklü bir thread için bir MCP sunucusunun başlatma durumu değiştiğinde yayınlanır. |  |
| `mcpServer/event/stream/notification` | Başlatılmış bir MCP olay akışının bildirimini iletir. | deneysel |
| `app/list/updated` | Uygulama (connector) listesi değiştiğinde yayınlanır. Sunucu bunu experimentalApi kapısı olmadan gönderir. | deneysel (yorumda) |

#### Dosya sistemi, süreç ve arama (5)

| Metot | Ne için | Durum |
|---|---|---|
| `process/outputDelta` | `process/spawn` akışının base64 stdout/stderr parçası. | deneysel |
| `process/exited` | `process/spawn` süreci çıktığında son çıkış durumunu bildirir. | deneysel |
| `fs/changed` | `fs/watch` ile izlenen bir yolda değişiklik olduğunda yayınlanır. |  |
| `fuzzyFileSearch/sessionUpdated` | Dosya arama oturumunda aktif sorgunun güncel eşleşmelerini bildirir. |  |
| `fuzzyFileSearch/sessionCompleted` | Dosya arama oturumunda bir sorgunun indeksleme ve eşleştirmesi bittiğinde yayınlanır. |  |

#### Yapılandırma içe aktarma ve uzaktan kontrol (3)

| Metot | Ne için | Durum |
|---|---|---|
| `remoteControl/status/changed` | Uzaktan kontrol bağlantı durumu veya uzak kimlik değiştiğinde yayınlanır. |  |
| `externalAgentConfig/import/progress` | Dış agent yapılandırması içe aktarma ilerlemesini bildirir. |  |
| `externalAgentConfig/import/completed` | Dış agent yapılandırması içe aktarması bittiğinde yayınlanır. |  |

#### Gerçek zamanlı (11)

| Metot | Ne için | Durum |
|---|---|---|
| `thread/realtime/started` | Thread'in gerçek zamanlı oturumu başladığında yayınlanır. | deneysel |
| `thread/realtime/itemAdded` | Gerçek zamanlı oturuma bir öğe eklendiğinde yayınlanır. | deneysel |
| `thread/realtime/item/started` | Gerçek zamanlı bir öğe başladığında yayınlanır. | deneysel |
| `thread/realtime/item/transcript/delta` | Gerçek zamanlı bir öğenin transkript parçasını iletir. | deneysel |
| `thread/realtime/item/completed` | Gerçek zamanlı bir öğe tamamlandığında yayınlanır. | deneysel |
| `thread/realtime/transcript/delta` | Gerçek zamanlı oturumun transkript parçasını iletir. | deneysel |
| `thread/realtime/transcript/done` | Gerçek zamanlı transkript bittiğinde yayınlanır. | deneysel |
| `thread/realtime/outputAudio/delta` | Gerçek zamanlı çıktı sesinin bir parçasını iletir. | deneysel |
| `thread/realtime/sdp` | Gerçek zamanlı oturum için SDP bilgisini iletir. | deneysel |
| `thread/realtime/error` | Gerçek zamanlı oturumda hata oluştuğunda yayınlanır. | deneysel |
| `thread/realtime/closed` | Gerçek zamanlı oturum kapandığında yayınlanır. | deneysel |

## 3. Sunucu → istemci istekleri (11, onaylar dahil)

Sunucu istemciden yanıt bekler; istemci `ServerResponse` ile cevaplamalıdır.

### Çekirdek (onaylar ve kullanıcı girdisi)

| Metot | Ekran | Ne için | Durum |
|---|---|---|---|
| `item/commandExecution/requestApproval` | Onaylar | Bir komut çalıştırılmadan önce onay ister. Yanıt: `accept`, `acceptForSession`, `decline`, `cancel` veya execpolicy değişikliği. Ağ erişimi için `networkApprovalContext` gelebilir. Yönetilen insan onayı kontrol noktasında `kind: humanCommand` ile gelir; bu akış `requiredHumanApproval` ve `experimentalApi` gerektirir. |  |
| `item/fileChange/requestApproval` | Onaylar | Bir dosya değişikliği uygulanmadan önce onay ister. Yanıt: `accept`, `acceptForSession`, `decline`, `cancel`. |  |
| `item/permissions/requestApproval` | Onaylar | Yerleşik `request_permissions` aracının istediği ağ veya dosya sistemi izinlerini onaylatır. Yanıtta yalnızca verilen alt küme bulunur. |  |
| `mcpServer/elicitation/request` | Onaylar | Bir MCP sunucusunun tur sırasında istediği form girdisini veya URL akışı onayını ister. Yanıt: `accept` ve içerik, ya da `decline`/`cancel`. |  |
| `item/tool/requestUserInput` | Onaylar | Bir araç çağrısı için kullanıcıya 1–3 kısa soru sorar. `autoResolutionMs` süresi dolunca otomatik çözülebilir. Sunucu bunu experimentalApi kapısı olmadan gönderir. | deneysel (yorumda) |

### Diğer

| Metot | Ne için | Durum |
|---|---|---|
| `item/tool/call` | İstemcinin yerelde çalıştırması gereken dinamik araç çağrısı. Araçlar `thread/start.dynamicTools` ile tanımlanır; o alan deneyseldir. |  |
| `account/chatgptAuthTokens/refresh` | Dış host uygulamanın sağladığı ChatGPT token'larının yenilenmesini ister. Kimlik modu iç kullanım için işaretlidir. | kararsız |
| `attestation/generate` | `requestAttestation` yeteneği açıksa, upstream `x-oai-attestation` için taze bir attestation sonucu üretmesini ister. |  |
| `currentTime/read` | İstemcinin sahip olduğu dış saat kaynağından güncel zamanı okur. | deneysel |
| `applyPatchApproval` | Eski API (`SendUserTurn`, `SendUserMessage`) ile başlatılan turlarda yama uygulama onayı ister. | kullanımdan kalkmış |
| `execCommandApproval` | Eski API ile başlatılan turlarda komut çalıştırma onayı ister. | kullanımdan kalkmış |

## Ek: Kısmen deneysel metotların deneysel istek alanları

Aşağıdaki metotlar kararlıdır. Yalnızca listelenen istek alanları `experimentalApi` gerektirir; alan açık değilse istek `<alan> requires experimentalApi capability` hatasıyla reddedilir. Alan adları wire (camelCase) biçimindedir; `mock` ile başlayan test alanları hariç tutulmuştur. `account/login/start` için liste, parametrenin (enum) varyantlarıdır. `thread/settings/update` tümüyle deneysel olduğu için burada yer almaz.

| Metot | Deneysel istek alanları |
|---|---|
| `thread/start` | `allowProviderModelFallback`, `runtimeWorkspaceRoots`, `permissions`, `multiAgentMode`, `historyMode`, `projectId`, `daybreakEnabled`, `environments`, `dynamicTools`, `selectedCapabilityRoots`, `experimentalRawEvents` |
| `thread/resume` | `history`, `path`, `runtimeWorkspaceRoots`, `permissions`, `initialTurnsPage` |
| `thread/fork` | `experimentalPredictionMode`, `experimentalDisablePredictionMcp`, `beforeTurnId`, `path`, `runtimeWorkspaceRoots`, `permissions`, `attachToSource`, `deferGoalContinuation` |
| `thread/metadata/update` | `projectId`, `daybreakEnabled` |
| `thread/list` | `projectId`, `parentThreadId`, `ancestorThreadId`, `attachedToThreadId` |
| `turn/start` | `responsesapiClientMetadata`, `responsesapiClientMetadataPatch`, `additionalContext`, `environments`, `runtimeWorkspaceRoots`, `permissions`, `collaborationMode`, `multiAgentMode`, `cyberAccessProgram` |
| `turn/steer` | `responsesapiClientMetadata`, `responsesapiClientMetadataPatch`, `additionalContext` |
| `account/login/start` | `chatgptAuthTokens`, `amazonBedrock`, `amazonBedrockAccessKeys` |
| `command/exec` | `permissionProfile` |

## Notlar

- Bu tablo, hangi metotların Makas'ta açık olacağını söylemez. Dış bağlantı riski taşıyan metotların (ör. `feedback/*`, `account/login/start`, `plugin/*`, `app/*`, `remoteControl/*`, `thread/realtime/*`) kapatılması K-06 ve K-17 kapsamında ayrıca kararlaştırılır.
- Sandbox dışında çalışan iki metot var: `thread/shellCommand` (tam erişim) ve `process/spawn` (Codex sandbox'ı olmadan). Bunların kullanımı için ayrı bir karar gerekir.
- `plugin/install` ve `plugin/uninstall` için depodaki README'de geliştirme notu yok. Resmi dokümanın bir kopyası bu iki metodu da "under development" olarak işaretliyor. Üretim kullanımından önce doğrulanmalı.
- Bu envanter henüz otomatik kontrol edilmiyor. K-09 sözleşme testini getirene kadar her upstream senkronunda (K-11) kaynakla elle karşılaştırılmalı.
