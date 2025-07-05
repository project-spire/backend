CREATE TABLE character_skills (
    character_id bigint not null references character(id),
    skill_id     bigint not null,

    level integer not null default 1,

    primary key(character_id, skill_id)
);
