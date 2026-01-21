-- creates a new tag color enum for each of the base 16 CSS colors
-- https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/named-color#standard_colors
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
    'color10',
    'color11',
    'color12',
    'color13',
    'color14',
    'color15',
    'color16',
    'color17',
    'color18',
    'color19',
    'color20',
);

ALTER TABLE tag
    ADD COLUMN color tag_color_enum DEFAULT 'color01' NOT NULL;

