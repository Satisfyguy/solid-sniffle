# models.py
from pydantic import BaseModel
from typing import List, Optional

# --- Sous-modèles ---
class RequestInfo(BaseModel):
    requestId: str
    timestamp: str
    source: str

class UserContext(BaseModel):
    userId: str
    permissions: List[str]
    language: str

class ChatMessage(BaseModel):
    role: str
    content: str

class SessionContext(BaseModel):
    sessionId: str
    chatHistory: List[ChatMessage]

# --- Le modèle de domaine (CE QUI EST NOUVEAU) ---
# Il doit correspondre exactement au "domain" de votre JSON
class DomainContext(BaseModel):
    app: str
    projectContextDocument: str  # Le champ pour votre long document

# --- Le Modèle MCP Principal ---
# C'est la structure complète de votre requête
class MCPRequest(BaseModel):
    mcpVersion: str
    requestInfo: RequestInfo
    user: UserContext
    session: SessionContext
    domain: DomainContext  # On utilise le modèle de domaine ci-dessus
    prompt: str          # La question principale
