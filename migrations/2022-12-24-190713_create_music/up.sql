CREATE TABLE music (
    id int NOT NULL AUTO_INCREMENT,
    title varchar(255) NOT NULL,
    artist varchar(255) NOT NULL,
    file_path varchar(255) NOT NULL,
    file_size int unsigned NOT NULL,
    PRIMARY KEY (id)
);