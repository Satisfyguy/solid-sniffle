from pydantic_settings import BaseSettings
from typing import List
import os
from pathlib import Path

class Settings(BaseSettings):
    # DeepSeek API Configuration
    DEEPSEEK_API_KEY: str = os.getenv("DEEPSEEK_API_KEY", "")
    DEEPSEEK_MODEL: str = "deepseek-chat"  # DeepSeek-V3
    DEEPSEEK_BASE_URL: str = "https://api.deepseek.com/v1"
    
    # Analysis Settings
    MAX_TOKENS: int = 128000  # 128K context
    TEMPERATURE: float = 0.2
    TIMEOUT: int = 300  # 5 minutes
    
    # File Patterns
    RUST_PATTERNS: List[str] = ["**/*.rs"]
    CONFIG_PATTERNS: List[str] = ["**/Cargo.toml", "**/Cargo.lock", "**/.env*"]
    
    # Output
    REPORT_DIR: Path = Path("reports")
    LOG_LEVEL: str = "INFO"
    
    class Config:
        env_file = ".env"
        case_sensitive = True

# Initialize settings
settings = Settings()

# Ensure report directory exists
settings.REPORT_DIR.mkdir(exist_ok=True)
