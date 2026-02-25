from fastapi import FastAPI

app = FastAPI(title="PurposeForge API")

@app.get("/")
def root():
    return {"message": "Welcome to PurposeForge API"}
