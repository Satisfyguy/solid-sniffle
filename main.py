# main.py
import os
import google.generativeai as genai
from fastapi import FastAPI, HTTPException
from models import MCPRequest  # On importe notre protocole !

# --- Configuration ---
app = FastAPI()

# Assurez-vous de définir votre clé API dans votre terminal
# export GEMINI_API_KEY="VOTRE_CLÉ_API_ICI"
try:
    genai.configure(api_key=os.environ["GEMINI_API_KEY"])
except KeyError:
    print("ERREUR: La variable d'environnement GEMINI_API_KEY n'est pas définie.")
    exit()

gemini_model = genai.GenerativeModel('gemini-1.5-pro-latest')

# --- L'Endpoint API ---
@app.post("/mcp-analyze")
async def handle_mcp_analysis(request: MCPRequest):
    """
    Cet endpoint reçoit le paquet MCP complet,
    l'analyse et l'envoie à Gemini.
    """
    
    # 1. Validation : 
    # FastAPI s'en est déjà occupé ! 
    # Si le JSON ne correspond pas à MCPRequest, il a déjà renvoyé une erreur 422.
    
    try:
        # 2. Préparation de l'historique pour Gemini
        gemini_history = []
        for msg in request.session.chatHistory:
            gemini_history.append({"role": msg.role, "parts": [msg.content]})

        # 3. Démarrer la session de chat avec l'historique
        chat_session = gemini_model.start_chat(history=gemini_history)

        # 4. Construire le prompt final
        # Nous combinons le document de contexte et la question de l'utilisateur
        # en UN SEUL message puissant.
        
        final_prompt_to_send = f"""
        Voici un document de contexte de projet très détaillé.
        Lis-le attentivement :

        --- DEBUT DU DOCUMENT DE CONTEXTE ---
        
        {request.domain.projectContextDocument}
        
        --- FIN DU DOCUMENT DE CONTEXTE ---

        Maintenant, en te basant sur ce document, réponds à ma question :
        
        {request.prompt}
        """

        # 5. Envoyer le tout à Gemini
        response = await chat_session.send_message_async(final_prompt_to_send)

        return {
            "source_request_id": request.requestInfo.requestId,
            "analysis_response": response.text
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
