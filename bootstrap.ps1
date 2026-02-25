# ============================
# PurposeForge Bootstrap Script
# ============================

$ErrorActionPreference = "Stop"

# Root directory of project
$ROOT = Split-Path -Parent $MyInvocation.MyCommand.Path
$BACKEND = Join-Path $ROOT "backend"

Write-Host "`n[+] Bootstrapping backend structure..." -ForegroundColor Cyan

# Create folders
$folders = @(
    "$BACKEND",
    "$BACKEND\app",
    "$BACKEND\app\api",
    "$BACKEND\app\core",
    "$BACKEND\app\models",
    "$BACKEND\app\schemas"
)

foreach ($folder in $folders) {
    if (-not (Test-Path $folder)) {
        New-Item -ItemType Directory -Path $folder | Out-Null
        Write-Host "    Created: $folder"
    }
}

# Create __init__.py files
$initFiles = @(
    "$BACKEND\__init__.py",
    "$BACKEND\app\__init__.py",
    "$BACKEND\app\api\__init__.py",
    "$BACKEND\app\core\__init__.py",
    "$BACKEND\app\models\__init__.py",
    "$BACKEND\app\schemas\__init__.py"
)

foreach ($file in $initFiles) {
    if (-not (Test-Path $file)) {
        New-Item -ItemType File -Path $file | Out-Null
        Write-Host "    Created: $file"
    }
}

# requirements.txt
@"
fastapi
uvicorn[standard]
sqlalchemy
pydantic
python-dotenv
"@ | Set-Content "$BACKEND\requirements.txt" -Encoding utf8

Write-Host "    Wrote: requirements.txt"

# main.py
@"
from fastapi import FastAPI

app = FastAPI(title="PurposeForge API")

@app.get("/")
def root():
    return {"message": "Welcome to PurposeForge API"}
"@ | Set-Content "$BACKEND\main.py" -Encoding utf8

Write-Host "    Wrote: main.py"

# database.py
@"
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, declarative_base

SQLALCHEMY_DATABASE_URL = "sqlite:///./purposeforge.db"

engine = create_engine(
    SQLALCHEMY_DATABASE_URL, connect_args={"check_same_thread": False}
)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
"@ | Set-Content "$BACKEND\app\core\database.py" -Encoding utf8

Write-Host "    Wrote: database.py"

# config.py
@"
from dotenv import load_dotenv
import os

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL", "sqlite:///./purposeforge.db")
"@ | Set-Content "$BACKEND\app\core\config.py" -Encoding utf8

Write-Host "    Wrote: config.py"

Write-Host "`n[SUCCESS] Backend bootstrap completed successfully." -ForegroundColor Green
Write-Host "`nNext step:" -ForegroundColor Yellow
Write-Host "  cd backend"
Write-Host "  python -m venv .venv"
Write-Host "  .\.venv\Scripts\activate"
Write-Host "  pip install -r requirements.txt"
Write-Host "  uvicorn main:app --reload"