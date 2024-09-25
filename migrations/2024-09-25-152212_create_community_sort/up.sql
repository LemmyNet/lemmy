-- Establish different community sort options from post sorting
CREATE TYPE community_sort_type_enum AS ENUM (
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
  'Controversial',
  'Scaled',
  'NameAsc',
  'NameDesc'
);