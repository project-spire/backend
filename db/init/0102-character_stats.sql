CREATE TABLE character_stats (
    character_id bigint references characters(id),

    -- Level
    level smallint not null default 0,
    exp   int      not null default 0,
    
    -- Core
    strength     smallint not null default 0,
    dexterity    smallint not null default 0,
    constitution smallint not null default 0,
    intelligence smallint not null default 0,

    -- Optional
    faith smallint default null
);

CREATE FUNCTION create_character_stat() RETURNS trigger AS $$
BEGIN
    INSERT INTO character_stats (character_id)
    VALUES (NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_character_stat_after_insert
AFTER INSERT ON characters
FOR EACH ROW
EXECUTE FUNCTION create_character_stat();