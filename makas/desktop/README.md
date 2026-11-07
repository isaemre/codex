# Makas Desktop (Electron)

Electron uygulaması bu klasörde olacak (Faz 2).

Kurulum notları:
- Bu proje kendi bağımsız `package.json`'ıyla kurulmalı. Repo kökündeki upstream pnpm workspace'ine karışmamalı.
- Ana süreç (`src/main`), paketlenen `makas` binary'sini `app-server` modunda stdio ile başlatır.
- Arayüz app-server'a doğrudan değil, `src/agent` altındaki ara katman üzerinden bağlanır (K-18, K-19).
- Electron fuse'ları kapatılır: RunAsNode, NODE_OPTIONS, inspect.
