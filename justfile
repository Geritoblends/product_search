# Justfile for testing the fridge AI backend

# Add products
add-apple:
    http POST localhost:3000/products name="organic apples"

add-banana:
    http POST localhost:3000/products name="ripe bananas"

add-milk:
    http POST localhost:3000/products name="whole milk"

add-bread:
    http POST localhost:3000/products name="sourdough bread"

add-chicken:
    http POST localhost:3000/products name="free range chicken breast"

add-spinach:
    http POST localhost:3000/products name="fresh baby spinach"

add-cheese:
    http POST localhost:3000/products name="aged cheddar cheese"

add-eggs:
    http POST localhost:3000/products name="free range eggs"

add-olive-oil:
    http POST localhost:3000/products name="extra virgin olive oil"

add-yogurt:
    http POST localhost:3000/products name="greek yogurt"

# List all
list:
    http GET localhost:3000/products

# Similar searches
similar-fruit:
    http POST localhost:3000/getSimilar query="something fruity"

similar-dairy:
    http POST localhost:3000/getSimilar query="dairy products"

similar-protein:
    http POST localhost:3000/getSimilar query="high protein food"

similar-healthy:
    http POST localhost:3000/getSimilar query="healthy greens"

# Update
update-apple:
    http PUT localhost:3000/products old_name="organic apples" new_name="red delicious apples"

# Delete
delete-banana:
    http DELETE localhost:3000/products name="ripe bananas"

# Delete all known products
clean:
    http POST localhost:3000/products/clear

# Seed all products at once
seed: add-apple add-banana add-milk add-bread add-chicken add-spinach add-cheese add-eggs add-olive-oil add-yogurt

seed-big:
    http POST localhost:3000/products name="organic apples"
    http POST localhost:3000/products name="ripe bananas"
    http POST localhost:3000/products name="whole milk"
    http POST localhost:3000/products name="sourdough bread"
    http POST localhost:3000/products name="free range chicken breast"
    http POST localhost:3000/products name="fresh baby spinach"
    http POST localhost:3000/products name="aged cheddar cheese"
    http POST localhost:3000/products name="free range eggs"
    http POST localhost:3000/products name="extra virgin olive oil"
    http POST localhost:3000/products name="greek yogurt"
    http POST localhost:3000/products name="strawberries"
    http POST localhost:3000/products name="blueberries"
    http POST localhost:3000/products name="raspberries"
    http POST localhost:3000/products name="watermelon"
    http POST localhost:3000/products name="mango"
    http POST localhost:3000/products name="pineapple"
    http POST localhost:3000/products name="avocado"
    http POST localhost:3000/products name="cherry tomatoes"
    http POST localhost:3000/products name="broccoli"
    http POST localhost:3000/products name="kale"
    http POST localhost:3000/products name="carrots"
    http POST localhost:3000/products name="sweet potatoes"
    http POST localhost:3000/products name="brown rice"
    http POST localhost:3000/products name="quinoa"
    http POST localhost:3000/products name="pasta"
    http POST localhost:3000/products name="canned chickpeas"
    http POST localhost:3000/products name="black beans"
    http POST localhost:3000/products name="lentils"
    http POST localhost:3000/products name="tofu"
    http POST localhost:3000/products name="salmon fillet"
    http POST localhost:3000/products name="tuna steak"
    http POST localhost:3000/products name="ground beef"
    http POST localhost:3000/products name="pork chops"
    http POST localhost:3000/products name="turkey breast"
    http POST localhost:3000/products name="bacon"
    http POST localhost:3000/products name="butter"
    http POST localhost:3000/products name="heavy cream"
    http POST localhost:3000/products name="parmesan cheese"
    http POST localhost:3000/products name="mozzarella"
    http POST localhost:3000/products name="almond milk"
    http POST localhost:3000/products name="oat milk"
    http POST localhost:3000/products name="rolled oats"
    http POST localhost:3000/products name="granola"
    http POST localhost:3000/products name="peanut butter"
    http POST localhost:3000/products name="almond butter"
    http POST localhost:3000/products name="honey"
    http POST localhost:3000/products name="dark chocolate"
    http POST localhost:3000/products name="orange juice"
    http POST localhost:3000/products name="sparkling water"
    http POST localhost:3000/products name="green tea"

clean-seed-big: clean seed-big

# Run all tests in sequence
test: clean-seed-big list similar-fruit similar-dairy similar-protein similar-healthy update-apple delete-banana list

similar-meat:
    http POST localhost:3000/getSimilar query="meat and fish"

similar-grains:
    http POST localhost:3000/getSimilar query="grains and carbs"

similar-sweet:
    http POST localhost:3000/getSimilar query="something sweet"

similar-drinks:
    http POST localhost:3000/getSimilar query="drinks and beverages"

similar-vegan:
    http POST localhost:3000/getSimilar query="vegan protein sources"

test-big: clean-seed-big list similar-fruit similar-dairy similar-protein similar-healthy similar-meat similar-grains similar-sweet similar-drinks similar-vegan
