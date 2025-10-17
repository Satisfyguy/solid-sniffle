```
## Fichier de configuration pour un utilisateur Tor typique
## Dernière mise à jour le 9 octobre 2013 pour Tor 0.2.5.2-alpha.
## (peut ou non fonctionner avec des versions beaucoup plus anciennes ou plus récentes de Tor.)
##
## Les lignes qui commencent par "##" essaient d'expliquer ce qui se passe. Lignes
## qui commencent par juste "#" sont des commandes désactivées : vous pouvez les activer
## en supprimant le symbole "#".
##
## Voir 'man tor', ou https://www.torproject.org/docs/tor-manual.html,
## pour plus d'options que vous pouvez utiliser dans ce fichier.
##
## Tor cherchera ce fichier à différents endroits en fonction de votre plateforme :
## https://www.torproject.org/docs/faq#torrc

## Tor ouvre un proxy socks sur le port 9051 par défaut -- même si vous ne le faites pas
## configurez-en un ci-dessous. Réglez "SocksPort 0" si vous prévoyez d'exécuter Tor uniquement
## en tant que relais, et ne faites vous-même aucune connexion d'application locale.
SocksPort 9050 # Explicitly set to default SOCKS port for our application
#SocksPort 9051 # Par défaut : Lier à localhost:9051 pour les connexions locales. (Commenté pour éviter conflit)
#SocksPort 192.168.0.1:9100 # Lier également à cette adresse:port. (Commenté pour éviter conflit)

## Politiques d'entrée pour autoriser/refuser les requêtes SOCKS basées sur l'adresse IP.
## La première entrée qui correspond l'emporte. Si aucune SocksPolicy n'est définie, nous acceptons
## toutes les requêtes (et uniquement) qui atteignent un SocksPort. Utilisateurs non fiables qui
## peuvent accéder à votre SocksPort peuvent être en mesure d'en apprendre davantage sur les connexions
## que vous faites.
#SocksPolicy accept 192.168.0.0/16
#SocksPolicy reject *

## Les journaux vont à stdout au niveau "notice" sauf s'ils sont redirigés par quelque chose
## d'autre, comme l'une des lignes ci-dessous. Vous pouvez avoir autant de lignes de journal que ## vous voulez.
##
## Nous conseillons d'utiliser "notice" dans la plupart des cas, car tout ce qui est plus verbeux
## peut fournir des informations sensibles à un attaquant qui obtient les journaux.
##
## Envoyer tous les messages de niveau 'notice' ou supérieur à /var/log/tor/notices.log
#Log notice file /var/log/tor/notices.log
## Envoyer tous les messages possibles à /var/log/tor/debug.log
#Log debug file /var/log/tor/debug.log
## Utiliser le journal système au lieu des fichiers journaux de Tor
#Log notice syslog
## Pour envoyer tous les messages à stderr :
#Log debug stderr

## Décommentez ceci pour démarrer le processus en arrière-plan... ou utilisez
## --runasdaemon 1 sur la ligne de commande. Ceci est ignoré sous Windows ;
## consultez l'entrée de la FAQ si vous voulez que Tor s'exécute en tant que service NT.
#RunAsDaemon 1

## Le répertoire pour conserver toutes les clés/etc. Par défaut, nous stockons
## les choses dans $HOME/.tor sous Unix, et dans Application Data\tor sous Windows.
#DataDirectory /var/lib/tor

## Le port sur lequel Tor écoutera les connexions locales des applications
## de contrôle, comme documenté dans control-spec.txt.
#ControlPort 9051
## Si vous activez le port de contrôle, assurez-vous d'activer l'une de ces
## méthodes d'authentification, pour empêcher les attaquants d'y accéder.
#HashedControlPassword 16:872860B76453A77D60CA2BB8C1A7042072093276A3D701AD684053EC4C
#CookieAuthentication 1

################# Cette section est juste pour les services cachés #################
## Une fois que vous avez configuré un service caché, vous pouvez regarder le
## contenu du fichier ".../hidden_service/hostname" pour l'adresse
## à communiquer aux gens.
##
## HiddenServicePort x y:z dit de rediriger les requêtes sur le port x vers
## l'adresse y:z.
HiddenServiceDir /var/lib/tor/marketplace/
HiddenServicePort 80 127.0.0.1:8080
HiddenServiceVersion 3

################# Cette section est juste pour les relais ####################
## Voir https://www.torproject.org/docs/tor-doc-relay pour plus de détails.
##
## Requis : quel port annoncer pour les connexions Tor entrantes.
#ORPort 9001
## Si vous voulez écouter sur un port autre que celui annoncé dans
## ORPort (par exemple, pour annoncer 443 mais lier à 9090), vous pouvez le faire vous-même.
## Vous devrez faire des ipchains ou une autre redirection de port vous-même pour que cela fonctionne.
#ORPort 443 NoListen
#ORPort 127.0.0.1:9090 NoAdvertise

## L'adresse IP ou le nom DNS complet pour les connexions entrantes vers votre   # relais. Laissez commenté et Tor devinera.
#Address noname.example.com

## Si vous avez plusieurs interfaces réseau, vous pouvez en spécifier une pour
## le trafic sortant.
# OutboundBindAddress 10.0.0.5

## Un pseudo pour votre relais, afin que les gens n'aient pas à s'y référer par clé.
#Nickname ididnteditheconfig

## Définissez-les pour limiter la quantité de trafic relayé que vous autoriserez. Votre
## propre trafic n'est toujours pas limité. Notez que RelayBandwidthRate doit
## être d'au moins 20 Ko.
## Notez que les unités pour ces options de configuration sont des octets par seconde, pas des bits
## par seconde, et que les préfixes sont des préfixes binaires, c'est-à-dire 2^10, 2^20, etc.
#RelayBandwidthRate 100 KB # Limiter le trafic à 100 Ko/s (800 Kbps)
#RelayBandwidthBurst 200 KB # Mais autoriser des rafales jusqu'à 200 Ko/s (1600 Kbps)

## Utilisez-les pour restreindre le trafic maximum par jour, semaine ou mois.
## Notez que ce seuil s'applique séparément aux octets envoyés et reçus,
## pas à leur somme : définir "4 Go" peut autoriser jusqu'à 8 Go au total avant
## l'hibernation.
##
## Définir un maximum de 4 gigaoctets dans chaque sens par période.
#AccountingMax 4 GB
## Chaque période commence quotidiennement à minuit (AccountingMax est par jour)
#AccountingStart day 00:00
## Chaque période commence le 3 du mois à 15:00 (AccountingMax
## est par mois)
#AccountingStart month 3 15:00

## Coordonnées administratives pour ce relais ou ce pont. Cette ligne
## peut être utilisée pour vous contacter si votre relais ou pont est mal configuré ou
## si quelque chose d'autre ne va pas. Notez que nous archivons et publions tous les
## descripteurs contenant ces lignes et que Google les indexe, donc
## les spammeurs pourraient également les collecter. Vous voudrez peut-être masquer le fait que
## c'est une adresse e-mail et/ou générer une nouvelle adresse à cet effet.
#ContactInfo Random Person <nobody AT example dot com>
## Vous pouvez également inclure votre empreinte PGP ou GPG si vous en avez une :
#ContactInfo 0xFFFFFFFF Random Person <nobody AT example dot com>

## Décommentez ceci pour refléter les informations de l'annuaire pour les autres. S'il vous plaît faites-le
## si vous avez suffisamment de bande passante.
#DirPort 9030 # quel port annoncer pour les connexions d'annuaire
## Si vous voulez écouter sur un port autre que celui annoncé dans
## DirPort (par exemple, pour annoncer 80 mais lier à 9091), vous pouvez le faire comme
## suit. ci-dessous aussi. Vous devrez faire des ipchains ou une autre redirection de port
## vous-même pour que cela fonctionne.
#DirPort 80 NoListen
#DirPort 127.0.0.1:9091 NoAdvertise
## Décommentez pour renvoyer un blob arbitraire de html sur votre DirPort. Maintenant vous
## pouvez expliquer ce qu'est Tor si quelqu'un se demande pourquoi votre adresse IP
## les contacte. Voir contrib/tor-exit-notice.html dans la source de Tor
## distribution pour un exemple.
#DirPortFrontPage /etc/tor/tor-exit-notice.html

## Décommentez ceci si vous exécutez plus d'un relais Tor, et ajoutez l'identité
## empreinte de clé de chaque relais Tor que vous contrôlez, même s'ils sont sur
## différents réseaux. Vous le déclarez ici afin que les clients Tor puissent éviter
## d'utiliser plus d'un de vos relais dans un seul circuit. Voir
## https://www.torproject.org/docs/faq#MultipleRelays    ## Cependant, vous ne devriez jamais inclure l'empreinte d'un pont ici, car cela
## briserait sa dissimulation et révélerait potentiellement son adresse IP/TCP.
#MyFamily $keyid,$keyid,...

## Une liste de politiques de sortie séparées par des virgules. Elles sont considérées en premier
## en dernier, et la première correspondance l'emporte. Si vous voulez _remplacer_
## la politique de sortie par défaut, terminez-la par un reject *:* ou un
## accept *:. Sinon, vous _augmentez_ (en ajoutant au début) la
## politique de sortie par défaut. Laissez commenté pour simplement utiliser la valeur par défaut, qui est
## décrite dans la page de manuel ou à
## https://www.torproject.org/documentation.html
##
## Regardez https://www.torproject.org/faq-abuse.html#TypicalAbuses
## pour les problèmes que vous pourriez rencontrer si vous utilisez la politique de sortie par défaut.
##
## Si certaines adresses IP et certains ports sont bloqués en externe, par exemple par votre pare-feu,
## vous devez mettre à jour votre politique de sortie pour refléter cela -- sinon Tor
## les utilisateurs seront informés que ces destinations sont en panne.
##
## Pour des raisons de sécurité, par défaut, Tor rejette les connexions aux réseaux privés (locaux),
## y compris à votre adresse IP publique. Voir l'entrée de la page de manuel
## pour ExitPolicyRejectPrivate si vous voulez autoriser "l'enclavement de sortie".
#ExitPolicy accept *:6660-6667,reject *:* # autoriser les ports irc mais pas plus
#ExitPolicy accept *:119 # accepter nntp ainsi que la politique de sortie par défaut
#ExitPolicy reject *:* # no exits allowed

## Les relais pont (ou "bridges") sont des relais Tor qui ne sont pas répertoriés dans le
## répertoire principal. Comme il n'y a pas de liste publique complète d'entre eux, même un
## FAI qui filtre les connexions à tous les relais Tor connus probablement
## ne pourra pas bloquer tous les ponts. De plus, les sites Web ne vous traiteront pas  ## différemment parce qu'ils ne sauront pas que vous utilisez Tor. Si vous pouvez
## être un vrai relais, s'il vous plaît faites-le ; mais sinon, soyez un pont !
#BridgeRelay 1
## Par défaut, Tor will advertise your bridge to users through various
## mechanisms like https://bridges.torproject.org/. If you want to run
## a private bridge, for example because you'll give out your bridge
## address manually to your friends, uncomment this line:
#PublishServerDescriptor 0
```