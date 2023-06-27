create or replace function row_number_partion(
    category numeric,
    score numeric
)
returns integer as $$
begin
  return row_number() over (partition by category order by score desc)::integer;
end; $$
LANGUAGE plpgsql;

-- Update the enums
ALTER TYPE sort_type_enum ADD VALUE 'BestAll';
ALTER TYPE sort_type_enum ADD VALUE 'BestYear';
ALTER TYPE sort_type_enum ADD VALUE 'BestThreeMonth';
ALTER TYPE sort_type_enum ADD VALUE 'BestSixMonth';
ALTER TYPE sort_type_enum ADD VALUE 'BestNineMonth';
ALTER TYPE sort_type_enum ADD VALUE 'BestMonth';
ALTER TYPE sort_type_enum ADD VALUE 'BestWeek';
ALTER TYPE sort_type_enum ADD VALUE 'BestDay';
ALTER TYPE sort_type_enum ADD VALUE 'BestTwelveHour';
ALTER TYPE sort_type_enum ADD VALUE 'BestSixHour';
ALTER TYPE sort_type_enum ADD VALUE 'BestHour';
