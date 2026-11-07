# Makas: Vizyon

> Makas, Codex tabanlı, sadece şirket içi LLM'e bağlanan ve upstream'le kolay senkron kalacak şekilde geliştirilen kurumsal kodlama agent'ıdır.

## Neden
Şirketimizde yazılımcılar ve analistler kodlama agent'larını giderek daha çok kullanıyor. Bugün herkes farklı araçlar kullanıyor ve bu araçların bir kısmı kodumuzu şirket dışındaki modellere gönderebiliyor. IT olarak harici agent'ları yasaklamak istiyoruz, ama bunu ekiplerin elinden aracı alıp boşluk bırakarak değil, önce güvenli ve en az onlar kadar iyi bir alternatif sunarak yapmak istiyorum.

## Makas nedir
- Açık kaynak OpenAI Codex'in kurum içi fork'u.
- Sadece şirket içi LLM'imizle çalışır. Bu kilit agent'ın çekirdeğine gömülüdür; kullanıcı değiştiremez.
- Kullanıcıların kendi bilgisayarında çalışır.
- Yazılımcılar ve analistler için bir masaüstü arayüzü (Electron) vardır.

## İlkeler
Kararlar çatıştığında öncelik sırası aşağıdaki gibidir.

1. **Güvenlik.** Şirket dışındaki hiçbir modele bağlanılmaz. Yeni bir dış bağlantı eklenmez. Kilit kullanıcı ayarında değil, çekirdekte durur.
2. **Sürdürülebilirlik.** Fork upstream'e yakın tutulur:
   - Kendi kodumuz kendi klasörlerimizde durur; upstream'de sadece işaretli tek satırlık kancalar olur.
   - Upstream kodu silinmez, gerekirse tek noktadan kapatılır.
   - Her değişikliğin testi vardır.
   - Bir şey config ile çözülebiliyorsa kod yazılmaz.
3. **Kullanıcı deneyimi.** Makas, kullanıcıya harici araçları aratmayacak kadar iyi olmalı. Aksi hâlde yasak işe yaramaz.
4. **Özellik.** Yeni özellik ancak ilk üç ilke sağlandıktan sonra eklenir.

## Kullanıcılar
- **Pilot:** Önce kendi ekibim (8 yazılımcı ve 2 analist), sonra tüm şirket.
- **Ortam:** Kullanıcıların çoğu Windows, birkaçı macOS kullanıyor. Çoğu local admin değil.

## Kapsam
**İlk etapta yapılacaklar (Faz 0–2)**
- Codex'in şirket içi LLM ile doğrulanması, gateway, yönetilen config, pilot.
- Codex fork'u: LLM kilidi, istenmeyen özelliklerin kapatılması, Makas adı ve imzalı sürümler.
- Electron masaüstü arayüzü.

**Şimdilik kapsam dışı**
- Agent'ın çalıştırdığı komutların ağ erişimini kontrol etmek. Codex'in sandbox'ı bunu sağlıyorsa ek bir kazanç olarak görüyorum.
- Merkezi sunucu modu, mobil ve web istemcileri.
- IDE entegrasyonu (ACP).
- Şirket içi LLM dışında herhangi bir model.

## Başarıyı nasıl anlayacağız
- Egress testi her sürümde yeşil; şirket dışına hiç LLM trafiği gitmiyor.
- Pilot ekip günlük işinde Makas'ı kullanıyor ve harici bir agent'a ihtiyaç duymuyor.
- Bir upstream senkronu birkaç saatlik bir iş; birkaç günlük bir proje değil.
- Bilgi güvenliğinin onayı alınmış durumda.
- Canlıya çıkıştan sonra harici agent'lar engellenmiş ve kullanıcılar Makas'a geçmiş durumda.

## Yol haritası
- **Faz 0: Doğrulama ve temel altyapı.** Şirket içi LLM ile uçtan uca test, sandbox testi, gateway, yönetilen config, kendi ekibimle pilot.
- **Faz 1: Fork ve kilit.** LLM kilidi, kapatmalar, egress testi, Makas adı ve imzalı sürümler.
- **Faz 2: Masaüstü arayüzü.** app-server üzerine Electron uygulaması.
- **Canlı:** Faz 2 bittikten sonra tüm şirkete açılış. Harici LLM erişimi network'te kapatılır ve onaysız agent'lar engellenir.
- **Sonrası:** Bitbucket, Jira ve task uygulaması entegrasyonları, otomatik kod incelemesi, refinement asistanı, IDE entegrasyonu. Bunlar ihtiyaca göre seçilecek.

## Nasıl geliştiriyoruz
- Makas'ı Claude Code ile geliştiriyorum.
- Kurallar `CLAUDE.md` dosyasında, kararlar `makas/docs/KARARLAR.md` dosyasında.
- Bu dosya "neden" ve "ne" sorularını cevaplar; kararlar dosyası her kararın ne ve neden olduğunu yazar.