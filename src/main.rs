use std::error::Error;
use std::fs;
use std::ops::RangeBounds;
use std::path::Path;
use std::str::FromStr;
use heed::{Database, EnvOpenOptions};
use heed::byteorder::BigEndian;
use heed::types::{ByteSlice, I32};
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
    let db1: Database<ByteSlice, ByteSlice> = env.open_database(&wtxn, Some("sat_to_id"))?.unwrap();
    let db2: Database<I32<BigEndian>, ByteSlice> = env.open_database(&wtxn, Some("idx_to_id"))?.unwrap();

    db1.clear(&mut wtxn)?;
    wtxn.commit()?;

    let mut total = vec![];
    let mut total2 = vec![];
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
        total2.push((i, inscription_id));
    }

    let mut wtx = env.write_txn()?;

    total.clone().into_iter().for_each(|(p, i)| {
            db1.put(&mut wtx, &p.store(), &i.store()).unwrap();
    });

    total2.clone().into_iter().for_each(|(idx, i)| {
        db2.put(&mut wtx, &idx, &i.store()).unwrap();
    });

    wtx.commit().unwrap();

    let rtx =env.read_txn()?;

    // let start = total.first().unwrap().clone().0.store();
    let start = total.first().unwrap().clone().0.store();
    // let start = binding.as_slice();

    // let end = total.last().unwrap().clone().0.store();
    let end = total.last().unwrap().clone().0.store();
    // let end = binding.as_slice();
    // let range = db.range(&rtx, &(&start..=&end))?;


    let mut rets = vec![];
    for range in  db1.range(&rtx, &RangeInclusiveArray(start, end))? {
        let (k,v) = range?;
        let sat_point = SatPoint::load(k.try_into().unwrap());
        let inscription_id = InscriptionId::load(v.try_into().unwrap());
        rets.push((sat_point, inscription_id))
    }
    assert_eq!(total, rets);

    let mut rets2 = vec![];
    for range in db2.range(&rtx, &(0..=9))? {
        let (k,v) = range?;
        let inscription_id = InscriptionId::load(v.try_into().unwrap());
        rets2.push((k, inscription_id))
    }
    assert_eq!(total2, rets2);
    println!("rets2: {:?}", rets2);
    Ok(())
}

struct RangeInclusiveArray<const N: usize>([u8; N], [u8; N]);

impl<const N: usize> RangeBounds<[u8]> for RangeInclusiveArray<N> {
    fn start_bound(&self) -> std::ops::Bound<&[u8]> {
        std::ops::Bound::Included(&self.0)
    }

    fn end_bound(&self) -> std::ops::Bound<&[u8]> {
        std::ops::Bound::Included(&self.1)
    }
}