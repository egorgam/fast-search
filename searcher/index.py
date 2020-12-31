import meilisearch
import csv

client = meilisearch.Client("http://127.0.0.1:7700", "masterKey")

index = client.index('tracks')

documents = []
with open("./../store/raw/tracks.csv") as file:
    reader = csv.reader(file, delimiter=',')
    for row in reader:
        documents.append({"track_id": row[0], "title": row[2]})

index.add_documents(documents)
