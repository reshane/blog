-- @COMMAND
DROP TABLE IF EXISTS recipe_ingredient;
-- @COMMAND
DROP TABLE IF EXISTS recipe;
-- @COMMAND
DROP TABLE IF EXISTS ingredient;
-- @COMMAND
DROP TABLE IF EXISTS post;
-- @COMMAND
-- recipe table
CREATE TABLE recipe(
    id SERIAL PRIMARY KEY,
    publish_date DATE NOT NULL DEFAULT CURRENT_DATE,
    dish_name TEXT,
    instructions TEXT
);
-- @COMMAND
-- ingredient table
CREATE TABLE ingredient(
    id SERIAL PRIMARY KEY,
    name TEXT
);
-- @COMMAND
-- recipe_ingredient assoc. table
CREATE TABLE recipe_ingredient(
    id SERIAL PRIMARY KEY,
    recipe_id INT,
    ingredient_id INT,
    CONSTRAINT fk_recipe FOREIGN KEY(recipe_id) REFERENCES recipe(id),
    CONSTRAINT fk_ingredient FOREIGN KEY(ingredient_id) REFERENCES ingredient(id)
);
-- @COMMAND
-- posts table
CREATE TABLE post(
    id SERIAL PRIMARY KEY,
    publish_date DATE NOT NULL DEFAULT CURRENT_DATE,
    title TEXT,
    body TEXT
);
