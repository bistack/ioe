#include <stddef.h>
#include <stdint.h>

struct db;
typedef struct db db_t;
int db_open(const char* dbname, db_t** newdb);
int db_close(db_t* db);
int db_read(db_t* db, uint32_t cid, uint32_t off4k, char* buf, size_t buflen);
int db_write(db_t* db, uint32_t cid, uint32_t off4k, const char* buf, size_t buflen);
