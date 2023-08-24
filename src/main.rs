use std::error::Error;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use heed::{Database, EnvOpenOptions};
use heed::types::ByteSlice;
use ord::inscription_id::InscriptionId;
use ord::SatPoint;
use ord::index::entry::Entry;

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("target").join("heed.mdb");

    fs::create_dir_all(&path)?;

    let env = EnvOpenOptions::new()
        .map_size(10 * 1024 * 1024)
        .max_dbs(3000)
        .open(path)?;

    let mut wtxn = env.write_txn()?;
    let db: Database<ByteSlice, ByteSlice> = env.create_database(&mut wtxn, None)?;

    db.clear(&mut wtxn)?;
    wtxn.commit()?;

    let mut total = vec![];
    for i in 0..10 {
        let inscription_id = InscriptionId::from_str(&format!(
            "a3c05702d7bc333a294b8dafef51b7614b83bbee001b14aa714628420f6c03ffi{i}"
        ))
            .unwrap();
        let sat_point = SatPoint::from_str(&format!(
            "a3c05702d7bc333a294b8dafef51b7614b83bbee001b14aa714628420f6c03ff:0:{i}"
        ))
            .unwrap();
        total.push((sat_point, inscription_id));
    }

    let mut wtx = env.write_txn()?;

    total.clone().into_iter().for_each(|(p, i)| {
            db.put(&mut wtx, &p.store(), &i.store()).unwrap();
    });

    wtx.commit().unwrap();

    let rtx =env.read_txn()?;

    let start = total.first().unwrap().clone().0.store();
    // let start = binding.as_slice();

    let end = total.last().unwrap().clone().0.store();
    // let end = binding.as_slice();

    let rets: Vec<(&[u8], _)> = db.range(&rtx, &(&start..=&end))?.collect()?;

    let re_total = rets
        .into_iter()
        .enumerate()
        .map(|(idx, value)| {
            let id = InscriptionId::load(value.1.try_into().unwrap());
            (total[idx].0.clone(), id)
        })
        .collect::<Vec<(SatPoint, InscriptionId)>>();
    assert_eq!(total, re_total);

    Ok(())
}