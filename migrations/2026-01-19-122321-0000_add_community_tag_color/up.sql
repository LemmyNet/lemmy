-- creates a new tag color enum
CREATE TYPE tag_color_enum AS ENUM (
    'color01',
    'color02',
    'color03',
    'color04',
    'color05',
    'color06',
    'color07',
    'color08',
    'color09',
    'color10'
);

ALTER TABLE tag
    ADD COLUMN color tag_color_enum DEFAULT 'color01' NOT NULL;

