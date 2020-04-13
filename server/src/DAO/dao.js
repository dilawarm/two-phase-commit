// @flow

module.exports = class Dao {
    pool: any;
  
    constructor(pool: any) {
      // Dependency Injection
      this.pool = pool;
    }
  
    query(sql: string, params: Array<mixed>, callback: function) {
      this.pool.getConnection((err: Error, connection: function) => {
        console.log("dao: connected to database");
        if (err) {
          console.log("dao: error connecting");
          callback(500, { error: "feil ved ved oppkobling" });
        } else {
          //console.log("dao: running sql: " + sql);
          
          connection.query(sql, params, (err: string, rows: []) => {
            connection.release();
            if (err) {
              console.log(err);
              callback(500, { error: "error querying" });
            } else {
              //console.log("dao: returning rows");
              callback(200, rows);
            }
          });
        }
      });
    }
  };