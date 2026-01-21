-- creates a new tag color enum for each of the base 16 CSS colors
-- https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/named-color#standard_colors
CREATE TYPE tag_color_enum AS ENUM (
    'gray',
    'maroon',
    'red',
    'purple',
    'fuchsia',
    'green',
    'lime',
    'yellow',
    'navy',
    'aqua'
);

ALTER TABLE tag
    ADD COLUMN color tag_color_enum DEFAULT 'green' NOT NULL;

