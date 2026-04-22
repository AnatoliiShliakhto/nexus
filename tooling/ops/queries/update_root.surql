BEGIN TRANSACTION;

LET $pass = IF $password = NONE OR $PASSWORD = NULL {
    NONE
} ELSE {
    crypto::argon2::generate($password)
};

UPSERT account:root SET login = $login, password = $pass, status = 'ACTIVE';

COMMIT TRANSACTION;