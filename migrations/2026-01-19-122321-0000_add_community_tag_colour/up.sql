-- creates a new tag color enum for each of the base 16 CSS colors
-- https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/named-color#standard_colors
CREATE TYPE tag_color_enum AS ENUM (
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
    ADD COLUMN color tag_color_enum DEFAULT 'Silver';

