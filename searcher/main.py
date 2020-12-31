from fastapi import FastAPI
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware
import meilisearch

client = meilisearch.Client("http://127.0.0.1:7700", "masterKey")

index = client.index('tracks')

app = FastAPI()

origins = [
    "http://localhost:8080",
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/search")
async def read_item(query: str = None) -> JSONResponse:
    if query:
        result = index.search(query)
        return JSONResponse(content={"result": result["hits"]})
    else:
        return JSONResponse(content={"result": None})
