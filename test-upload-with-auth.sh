#!/bin/bash

echo "🧅 Test d'upload d'images avec authentification"
echo "=============================================="

# Créer une image de test
echo "📸 Création d'une image de test..."
echo "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==" | base64 -d > test-image.png

# Créer un cookie de session (simulation)
echo "🍪 Simulation d'une session authentifiée..."

# Test de l'upload avec session simulée
echo "📤 Test d'upload d'images..."
UPLOAD_RESPONSE=$(curl -s -X POST \
  -F "images=@test-image.png" \
  -H "Cookie: session=test-session" \
  http://localhost:8080/api/listings/test-listing-id/images)

echo "Réponse: $UPLOAD_RESPONSE"

# Nettoyage
rm -f test-image.png

echo ""
echo "✅ Test terminé !"
echo ""
echo "🎯 Pour tester complètement dans le navigateur :"
echo "1. Ouvrez http://localhost:8080"
echo "2. Connectez-vous en tant que vendeur"
echo "3. Créez un listing"
echo "4. Uploadez des images via l'interface drag-and-drop"
echo "5. Les images devraient maintenant s'afficher !"
