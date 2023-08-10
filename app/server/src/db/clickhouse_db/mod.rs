use clickhouse_rs::Pool;
pub fn clickhouse_pool_builder(db_user:&str,db_password:&str,db_host:&str,db_port:&u16,db_name:&str) -> clickhouse_rs::Pool{

    let database_url = format!(
        "clickhouse://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );
    let pool = Pool::new(database_url);
    // let pool = mysql::Pool::new(builder).unwrap();
    pool
}
