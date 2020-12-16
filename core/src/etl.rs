use tantivy::schema::*;
use tantivy::Index;
use tantivy::tokenizer::NgramTokenizer;

use std::path::Path;
use std::fs::File;

const TOKENIZATOR_NAME: &str = "ngram";

fn apply_tokenizator(index: tantivy::Index) -> tantivy::Index {
    index.tokenizers()
        .register(TOKENIZATOR_NAME, NgramTokenizer::new(1, 5, false));
    return index
}

pub fn init_schema() -> tantivy::schema::Schema {
    let mut schema_builder = Schema::builder();
    let text_field_indexing = TextFieldIndexing::default()
                                                .set_tokenizer(TOKENIZATOR_NAME)
                                                .set_index_option(IndexRecordOption::WithFreqsAndPositions);

    let text_options = TextOptions::default()
                                    .set_indexing_options(text_field_indexing)
                                    .set_stored();

    schema_builder.add_text_field("track_id", STRING | STORED);
    schema_builder.add_text_field("name", text_options);
    return schema_builder.build()
}

pub fn init_index(schema: &tantivy::schema::Schema) -> tantivy::Index {
    let index_path = "./../store/index";
    if Path::new(&[index_path, "meta.json"].join("/")).exists(){
        let index = Index::open_in_dir(&index_path).unwrap();
        return apply_tokenizator(index)
    }
    else {
        let index = apply_tokenizator(Index::create_in_dir(&index_path, schema.clone()).unwrap());
        let mut index_writer = index.writer(50_000_000).unwrap();
        let track_id = schema.get_field("track_id").unwrap();
        let name = schema.get_field("name").unwrap();

        let file = File::open("./../store/raw/tracks.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.records() {
            let record = result.unwrap();
            let track_id_csv = &record[0];
            let name_csv = &record[2];
            index_writer.add_document(doc!(
                track_id => track_id_csv,
                name => name_csv.to_lowercase()
                ));
        }
        index_writer.commit().unwrap();
        return index
    }
}