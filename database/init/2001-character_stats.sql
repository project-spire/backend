CREATE TYPE CharacterStatType AS ENUM (
    'Level',
    'Experience',
    'Strength',
    'Dexterity',
    'Constitution',
    'Intelligence',
    'Faith'
);

CREATE TABLE character_stats (
    character_id bigint            not null references characters(id),
    type         CharacterStatType not null,
    value        int               not null default 0,

    primary key(character_id, type)
);
