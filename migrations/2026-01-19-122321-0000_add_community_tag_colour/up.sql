-- creates a new tag colour enum for each of the base 16 CSS colours
-- https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/named-color#standard_colors
CREATE TYPE tag_colour_enum AS ENUM (
    'Black',
    'Silver',
    'Gray',
    'White',
    'Maroon',
    'Red',
    'Purple',
    'Fuchsia',
    'Green',
    'Lime',
    'Olive',
    'Yellow',
    'Navy',
    'Blue',
    'Teal',
    'Aqua'
);

ALTER TABLE tag
    ADD COLUMN colour tag_colour_enum DEFAULT 'Silver';

