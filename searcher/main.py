from fastapi import FastAPI
from fastapi.responses import JSONResponse
from fastapi.middleware.cors import CORSMiddleware
import requests

CORE_URL = "http://127.0.0.1:8080/search"

app = FastAPI()

origins = [
    "http://localhost:8081",
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/search")
def read_item(query: str = None) -> JSONResponse:
    if query is None:
        return JSONResponse(content={"result": None})
    else:
        response = requests.get(CORE_URL, params={"query": query})
        return JSONResponse(content=response.json())
