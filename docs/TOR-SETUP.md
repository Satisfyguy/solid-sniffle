# 🧅 Tor Setup Guide - Monero Marketplace

**Classification:** PUBLIC  
**Threat Level:** HIGH  
**Deployment:** Tor Hidden Service Production

---

## 🚀 **Installation Tor**

### **Windows**
```powershell
# 1. Télécharger Tor Browser
# https://www.torproject.org/download/

# 2. Extraire et installer
# C:\tor-browser\

# 3. Lancer Tor
C:\tor-browser\Browser\TorBrowser\Tor\tor.exe

# 4. Vérifier que Tor tourne
Get-Process tor
```

### **Linux**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install tor

# CentOS/RHEL
sudo yum install tor

# Arch Linux
sudo pacman -S tor

# Lancer Tor
sudo systemctl start tor
sudo systemctl enable tor
```

### **macOS**
```bash
# Avec Homebrew
brew install tor

# Lancer Tor
brew services start tor
```

---

## ⚙️ **Configuration Tor**

### **Fichier de Configuration**
```bash
# /etc/tor/torrc (Linux/macOS)
# C:\Users\[user]\AppData\Roaming\tor\torrc (Windows)

# Configuration de base
SocksPort 9050
ControlPort 9051
CookieAuthentication 1

# Configuration pour marketplace
DataDirectory /var/lib/tor
Log notice file /var/log/tor/notices.log
Log info file /var/log/tor/debug.log

# Éviter les logs sensibles
SafeLogging 1
```

### **Configuration Avancée**
```bash
# Utiliser des bridges si Tor bloqué
UseBridges 1
Bridge obfs4 192.0.2.1:443 cert=abcd1234...
Bridge obfs4 192.0.2.2:443 cert=efgh5678...

# Configuration pour hidden service
HiddenServiceDir /var/lib/tor/marketplace
HiddenServicePort 80 127.0.0.1:8080
HiddenServicePort 443 127.0.0.1:8443

# Sécurité renforcée
StrictNodes 1
EnforceDistinctSubnets 1
```

---

## 🔧 **Configuration Monero + Tor**

### **Monero Daemon via Tor**
```bash
# Configuration monerod
monerod --testnet \
  --p2p-bind-ip 127.0.0.1 \
  --p2p-bind-port 18080 \
  --rpc-bind-ip 127.0.0.1 \
  --rpc-bind-port 18081 \
  --proxy 127.0.0.1:9050 \
  --anonymous-inbound 127.0.0.1:18080,25 \
  --data-dir ./testnet
```

### **Monero Wallet RPC via Tor**
```bash
# Configuration monero-wallet-rpc
monero-wallet-rpc --testnet \
  --wallet-file buyer \
  --password "" \
  --rpc-bind-ip 127.0.0.1 \
  --rpc-bind-port 18082 \
  --daemon-address 127.0.0.1:18081 \
  --proxy 127.0.0.1:9050
```

---

## 🧪 **Tests de Configuration**

### **Test de Connexion Tor**
```powershell
# Test IP leak
Invoke-RestMethod `
  -Uri "https://check.torproject.org/api/ip" `
  -Proxy "socks5://127.0.0.1:9050"

# Résultat attendu:
# {
#   "IP": "185.220.101.34",
#   "IsTor": true,
#   "Country": "DE"
# }
```

### **Test Monero RPC**
```powershell
# Test RPC via Tor
$body = @{
    jsonrpc = "2.0"
    id = "0"
    method = "get_version"
} | ConvertTo-Json

Invoke-RestMethod `
  -Uri "http://127.0.0.1:18082/json_rpc" `
  -Method Post `
  -ContentType "application/json" `
  -Body $body
```

### **Test de Performance**
```powershell
# Test de latence
Measure-Command {
    Invoke-RestMethod `
      -Uri "https://check.torproject.org/api/ip" `
      -Proxy "socks5://127.0.0.1:9050"
}

# Latence acceptable: < 30 secondes
```

---

## 🛡️ **Sécurité Tor**

### **Bonnes Pratiques**
- ✅ Utiliser des bridges si Tor bloqué
- ✅ Vérifier régulièrement la connexion
- ✅ Éviter les services clearnet
- ✅ Utiliser des circuits séparés
- ✅ Ne pas partager d'informations personnelles

### **Mauvaises Pratiques**
- ❌ Utiliser Tor pour des activités illégales
- ❌ Partager des informations personnelles
- ❌ Utiliser des services non-chiffrés
- ❌ Ignorer les avertissements de sécurité
- ❌ Utiliser des plugins non-Tor

---

## 🔍 **Monitoring Tor**

### **Statut Tor**
```powershell
# Vérifier que Tor tourne
Get-Process tor

# Vérifier les ports
netstat -an | Select-String "9050|9051"

# Vérifier les logs
Get-Content C:\Users\[user]\AppData\Roaming\tor\torrc
```

### **Métriques de Performance**
```powershell
# Latence des requêtes
Measure-Command { 
    Invoke-RestMethod -Uri "https://check.torproject.org/api/ip" -Proxy "socks5://127.0.0.1:9050" 
}

# Taux de succès
$success = 0
$total = 10
for ($i = 0; $i -lt $total; $i++) {
    try {
        Invoke-RestMethod -Uri "https://check.torproject.org/api/ip" -Proxy "socks5://127.0.0.1:9050"
        $success++
    } catch {
        # Échec
    }
}
Write-Host "Taux de succès: $($success/$total)"
```

---

## 🚨 **Dépannage**

### **Problèmes Courants**

#### **Tor ne se connecte pas**
```powershell
# Vérifier la configuration
Get-Content C:\Users\[user]\AppData\Roaming\tor\torrc

# Vérifier les logs
Get-Content C:\Users\[user]\AppData\Roaming\tor\torrc

# Redémarrer Tor
Get-Process tor | Stop-Process -Force
Start-Process "C:\tor-browser\Browser\TorBrowser\Tor\tor.exe"
```

#### **Connexion lente**
```bash
# Utiliser des bridges
UseBridges 1
Bridge obfs4 192.0.2.1:443 cert=abcd1234...

# Changer de circuit
echo "SIGNAL NEWNYM" | nc 127.0.0.1 9051
```

#### **RPC Monero inaccessible**
```powershell
# Vérifier que Monero tourne
Get-Process monero*

# Vérifier les ports
netstat -an | Select-String "18082"

# Redémarrer Monero
.\scripts\start-testnet.ps1
```

---

## 📊 **Optimisation**

### **Performance**
- Utiliser des bridges rapides
- Éviter les circuits lents
- Optimiser la configuration
- Utiliser des connexions persistantes

### **Sécurité**
- Utiliser des bridges obfs4
- Éviter les exit nodes suspects
- Changer de circuit régulièrement
- Monitorer les logs

---

## 🔗 **Ressources**

### **Documentation Officielle**
- [Tor Project](https://www.torproject.org/)
- [Tor Browser](https://www.torproject.org/download/)
- [Tor Manual](https://www.torproject.org/docs/tor-manual.html)

### **Outils**
- [Tor Metrics](https://metrics.torproject.org/)
- [Tor Status](https://torstatus.blutmagie.de/)
- [Tor Check](https://check.torproject.org/)

### **Communauté**
- [Tor Project Community](https://community.torproject.org/)
- [Tor IRC](https://www.torproject.org/contact.html#irc)

---

## 📝 **Checklist de Configuration**

### **Installation**
- [ ] Tor installé et configuré
- [ ] Ports 9050/9051 ouverts
- [ ] Configuration testée
- [ ] Logs configurés

### **Sécurité**
- [ ] Bridges configurés (si nécessaire)
- [ ] Hidden service configuré
- [ ] Logs sécurisés
- [ ] Monitoring activé

### **Tests**
- [ ] Test de connexion réussi
- [ ] Test IP leak réussi
- [ ] Test Monero RPC réussi
- [ ] Test de performance acceptable

### **Production**
- [ ] Configuration optimisée
- [ ] Monitoring en place
- [ ] Backup de configuration
- [ ] Documentation à jour

---

**Remember: Tor est un outil puissant, utilisez-le avec responsabilité. 🧅**
