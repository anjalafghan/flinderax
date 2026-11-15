-- One unit of money. Dates stored whole rupees while everything else stored paise; now it is
-- paise everywhere. Every Dates amount is multiplied by 100 (NULL stays NULL). The monthly date
-- budget and what a partner covers are settings kept as text: converted the same way.
UPDATE joy_activities SET est_cost = est_cost * 100;
UPDATE joy_seed_activities SET est_cost = est_cost * 100;
UPDATE joy_history SET est_cost = est_cost * 100, actual_cost = actual_cost * 100,
       cab_fare_0 = cab_fare_0 * 100, cab_est_0 = cab_est_0 * 100,
       cab_fare_1 = cab_fare_1 * 100, cab_est_1 = cab_est_1 * 100,
       from_savings = from_savings * 100;
UPDATE joy_month_budget SET base = base * 100, carry = carry * 100;
UPDATE joy_other_spending SET amount = amount * 100;
UPDATE joy_savings SET amount = amount * 100;
UPDATE joy_regulars SET usual_cost = usual_cost * 100;
UPDATE joy_regular_visits SET amount = amount * 100;
UPDATE joy_settings SET value = CAST(CAST(value AS INTEGER) * 100 AS TEXT)
 WHERE key IN ('monthly_budget', 'COVERS_0', 'COVERS_1') AND value GLOB '[0-9]*' AND value NOT GLOB '*[^0-9]*';
UPDATE joy_checkins SET budget = budget * 100;
