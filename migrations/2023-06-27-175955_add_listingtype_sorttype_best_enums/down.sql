-- update the default sort type
update local_user set default_sort_type = 'TopDay' where default_sort_type in (
    'BestAll',
    'BestYear',
    'BestNineMonth',
    'BestSixMonth',
    'BestThreeMonth',
    'BestMonth',
    'BestWeek',
    'BestDay',
    'BestTwelveHour',
    'BestSixHour',
    'BestHour'
);

-- rename the old enum
alter type sort_type_enum rename to sort_type_enum__;
-- create the new enum
CREATE TYPE sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'TopDay',
    'TopWeek',
    'TopMonth',
    'TopYear',
    'TopAll',
    'MostComments',
    'NewComments',
    'TopHour',
    'TopSixHour',
    'TopTwelveHour',
    'TopThreeMonths',
    'TopSixMonths',
    'TopNineMonths',
    );

-- alter all you enum columns
alter table local_user
  alter column default_sort_type type sort_type_enum using default_sort_type::text::sort_type_enum;

-- drop the old enum
drop type sort_type_enum__;

-- drop the function
drop function row_number_partion;