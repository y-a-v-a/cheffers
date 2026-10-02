// Cheffers Playground — wires a CodeMirror editor to the Chef interpreter
// compiled to WebAssembly. Everything runs client-side; no server involved.
//
// CodeMirror is bundled into editor.bundle.js at build time (see package.json),
// and the interpreter is the locally-built wasm-bindgen output in ./pkg/.
// The theme toggle is shared with the cookbook (../assets/theme.js); so is the
// Chef syntax tokenizer, which esbuild bundles in from ../assets/.

import { EditorView, basicSetup } from "codemirror";
import { Decoration, ViewPlugin, keymap } from "@codemirror/view";
import { Compartment, Prec, RangeSetBuilder } from "@codemirror/state";
import init, { run_chef } from "./pkg/cheffers_wasm.js";
import { escapeHtml, ansiToHtml } from "./ansi.js";
import { tokenizeChef } from "../assets/chef-syntax.js";

const EXAMPLES = {
  "hello-world": {
    label: "Hello World",
    source: `Hello World Souffle.

This recipe prints the immortal words "Hello world!", in a basically brute force way. It also makes a lot of food for one person.

Ingredients.
72 g haricot beans
101 eggs
108 g lard
111 cups oil
32 zucchinis
119 ml water
114 g red salmon
100 g dijon mustard
33 potatoes

Method.
Put potatoes into the mixing bowl. Put dijon mustard into the mixing bowl. Put lard into the mixing bowl. Put red salmon into the mixing bowl. Put oil into the mixing bowl. Put water into the mixing bowl. Put zucchinis into the mixing bowl. Put oil into the mixing bowl. Put lard into the mixing bowl. Put lard into the mixing bowl. Put eggs into the mixing bowl. Put haricot beans into the mixing bowl. Liquefy contents of the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "countdown-cake": {
    label: "Countdown Cake",
    source: `Countdown Cake.

A festive countdown recipe that counts from 5 down to 1. Perfect for New Year's Eve celebrations! The sugar rises from 1 to 5 as it is stacked into the mixing bowl, so the bowl pours out 5 4 3 2 1 when served.

Ingredients.
5 g flour
0 g sugar
1 g salt

Method.
Bake the flour. Put salt into the mixing bowl. Add sugar to the mixing bowl. Fold sugar into the mixing bowl. Put sugar into the mixing bowl. Bake the flour until baked. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "doubler-delight": {
    label: "Doubler Delight (input)",
    source: `Doubler Delight.

A simple dessert that takes any number and doubles it using the magic of addition. Try it with your favorite number!

Ingredients.
0 g sugar

Method.
Take sugar from refrigerator. Put sugar into the mixing bowl. Add sugar to the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
    input: "21",
  },
  "ratatouille": {
    label: "Remy's Ratatouille",
    source: `Remy's Ratatouille.

Anyone can cook! This recipe spells out "Ratatouille!" the way a certain rat would: every vegetable is measured so its quantity is the Unicode code point of one letter. The letters go into the bowl in reverse, because a mixing bowl is a stack and the last ingredient in is the first one served.

Ingredients.
82 g tomatoes
97 g aubergines
116 g zucchinis
111 ml olive oil
117 g red peppers
105 g onions
108 g garlic cloves
101 g herbes de provence
33 dashes tabasco

Method.
Put tabasco into the mixing bowl. Put herbes de provence into the mixing bowl. Put garlic cloves into the mixing bowl. Put garlic cloves into the mixing bowl. Put onions into the mixing bowl. Put red peppers into the mixing bowl. Put olive oil into the mixing bowl. Put zucchinis into the mixing bowl. Put aubergines into the mixing bowl. Put zucchinis into the mixing bowl. Put aubergines into the mixing bowl. Put tomatoes into the mixing bowl. Liquefy contents of the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "saturday-pancakes": {
    label: "Saturday Pancakes",
    source: `Saturday Pancakes.

A real Saturday-morning pancake recipe that is also a computer program: it works out how many pancakes the batter makes. The eggs are whisked away to nothing, the sugar is worked into the flour, and the butter is cut in, dividing the mixture into portions. The milk and salt rest in a second bowl and never reach the plate. Serve a stack of twelve.

Ingredients.
250 g flour
50 g sugar
25 g butter
3 eggs
500 ml milk
1 pinch salt

Method.
Whisk the eggs until whisked. Liquefy the butter. Put flour into the mixing bowl. Add sugar to the mixing bowl. Divide butter into the mixing bowl. Put milk into the 2nd mixing bowl. Put salt into the 2nd mixing bowl. Stir the mixing bowl for 2 minutes. Pour contents of the mixing bowl into the baking dish.

Serves 4.
`,
  },
  "gauss-layer-cake": {
    label: "Gauss's Layer Cake (input)",
    source: `Gauss's Layer Cake.

Young Gauss added the numbers 1 to 100 in seconds; this cake does it with layers. Tell the kitchen how many layers you want (the input) and each pass of the loop stacks the current layer count onto the icing sugar, counting down one layer at a time. Ten layers make 55, one hundred make 5050.

Ingredients.
0 g layers
0 g icing sugar

Method.
Take layers from the refrigerator. Put icing sugar into the mixing bowl. Stack the layers. Add layers to the mixing bowl. Stack the layers until stacked. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
    input: "10",
  },
  "tapas-times-table": {
    label: "Tapas Times Table (input)",
    source: `Tapas Times Table.

Ten guests arrive and each orders one more helping of tapas than nobody at all would: the kitchen prints the full times table of your chosen portion size, one line per guest. The second mixing bowl is the kitchen's scratch pad, where the running total is reduced by one portion per round, and the sparkling water (character 10) pours out as newlines.

Ingredients.
0 g portions
10 g guests
0 g serving
10 ml sparkling water

Method.
Take portions from the refrigerator. Put portions into the 2nd mixing bowl. Combine guests into the 2nd mixing bowl. Fold serving into the 2nd mixing bowl. Serve the guests. Put sparkling water into the mixing bowl. Put serving into the mixing bowl. Put serving into the 2nd mixing bowl. Remove portions from the 2nd mixing bowl. Fold serving into the 2nd mixing bowl. Serve the guests until served. Pour contents of the mixing bowl into the baking dish.

Serves 10.
`,
    input: "7",
  },
  "steak-au-poivre": {
    label: "Steak au Poivre (sous-chef)",
    source: `Bistro Steak au Poivre.

The steak is nothing without its sauce, so the head chef calls for a sous-chef. The sous-chef works in a copy of the kitchen with their own ingredient shelf, and whatever ends up in their first mixing bowl is handed back to the head chef. Six crushed peppercorns combined with seven spoons of cognac: the sauce knows the answer to everything.

Ingredients.
1 sirloin steak

Method.
Serve with peppercorn sauce. Pour contents of the mixing bowl into the baking dish.

Serves 1.

Peppercorn Sauce.

Ingredients.
6 g crushed peppercorns
7 ml cognac

Method.
Put crushed peppercorns into the mixing bowl. Combine cognac into the mixing bowl.
`,
  },
  "alphabet-soup": {
    label: "Alphabet Soup (shuffle)",
    source: `Alphabet Soup.

Pasta letters spelling ALPHABET go into the pot, and then the soup is given a good mix, which shuffles the order of everything in the bowl. What ladles out is an anagram of "alphabet" chosen by the interpreter's random shuffle. Natively every run stirs differently; in the playground the shuffle is seeded per page load.

Ingredients.
97 g letter a
98 g letter b
101 g letter e
104 g letter h
108 g letter l
112 g letter p
116 g letter t

Method.
Put letter t into the mixing bowl. Put letter e into the mixing bowl. Put letter b into the mixing bowl. Put letter a into the mixing bowl. Put letter h into the mixing bowl. Put letter p into the mixing bowl. Put letter l into the mixing bowl. Put letter a into the mixing bowl. Liquefy contents of the mixing bowl. Mix the mixing bowl well. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "melon-sorbet": {
    label: "Stirred Melon Sorbet",
    source: `Stirred Melon Sorbet.

The bowl starts out spelling MELON, top to bottom. Stirring rolls the top ingredient down into the bowl by the number of minutes stirred: two minutes bury the m two places, one more minute tucks the e just beneath the l, and the sorbet is served as LEMON. Same five ingredients, entirely different fruit.

Ingredients.
109 g melon balls
101 ml elderflower cordial
108 ml lime juice
111 g orange zest
110 g nutmeg

Method.
Put nutmeg into the mixing bowl. Put orange zest into the mixing bowl. Put lime juice into the mixing bowl. Put elderflower cordial into the mixing bowl. Put melon balls into the mixing bowl. Liquefy contents of the mixing bowl. Stir the mixing bowl for 2 minutes. Stir for 1 minute. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "bakers-dozen-scones": {
    label: "Baker's Dozen Scones",
    source: `Baker's Dozen Scones.

One instruction does all the measuring: adding the dry ingredients drops the sum of every dry ingredient into the bowl in a single move. Flour, sugar, baking powder and salt are dry and count; buttermilk and melted butter are liquid and stay out of the tally. The tray comes out of the oven with a baker's dozen.

Ingredients.
8 g self-raising flour
3 g caster sugar
1 pinch baking powder
1 pinch salt
150 ml buttermilk
55 ml melted butter

Method.
Add dry ingredients to the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 13.
`,
  },
  "overnight-oats": {
    label: "Overnight Oats",
    source: `Overnight Oats.

Some recipes end not with a bang but with a nap. Refrigerating prints the first baking dish and then stops the recipe on the spot, so the kitchen mumbles "zzz" and goes to sleep. The burnt toast (a suspicious 666) is queued up after the fridge closes and is never served, proving nothing runs past a Refrigerate.

Ingredients.
122 ml oat milk
666 g burnt toast

Method.
Put oat milk into the mixing bowl. Put oat milk into the mixing bowl. Put oat milk into the mixing bowl. Pour contents of the mixing bowl into the baking dish. Refrigerate for 1 hour. Put burnt toast into the mixing bowl. Pour contents of the mixing bowl into the baking dish.

Serves 1.
`,
  },
  "two-course-supper": {
    label: "Two Course Supper",
    source: `Two Course Supper.

A proper supper needs two courses, and this kitchen runs two mixing bowls and two baking dishes to plate them. The first bowl builds the word "soup" with a splash of sparkling water for the newline, the second bowl builds "cake", and each is poured into its own dish. Serving two dishes prints the first course, then dessert.

Ingredients.
115 ml stock
111 g onions
117 g udon noodles
112 g parsnips
99 g cocoa
97 g ground almonds
107 ml kirsch
101 g beaten eggs
10 ml sparkling water

Method.
Put sparkling water into the mixing bowl. Put parsnips into the mixing bowl. Put udon noodles into the mixing bowl. Put onions into the mixing bowl. Put stock into the mixing bowl. Liquefy contents of the mixing bowl. Put beaten eggs into the 2nd mixing bowl. Put kirsch into the 2nd mixing bowl. Put ground almonds into the 2nd mixing bowl. Put cocoa into the 2nd mixing bowl. Liquefy contents of the 2nd mixing bowl. Pour contents of the mixing bowl into the baking dish. Pour contents of the 2nd mixing bowl into the 2nd baking dish.

Serves 2.
`,
  },
  "mandelbrot-mille-feuille": {
    label: "Mandelbrot Mille-Feuille",
    source: `Mandelbrot Mille-Feuille.

A pastry of a thousand layers, each one a smaller copy of the whole. For every crumb on the plate the chef takes a cherry from the complex plane and folds it into a jam again and again (the jam is squared, the cherry is added back) until the jam boils over or the chef runs out of patience. Everything is baked in fixed-point arithmetic with 8192 grams of flour to the unit. The number of folds survived picks a flavour from a rack of 46 spices, from a bare plate for the impatient crumbs to a dense @ for the ones that never boil over. The spice rack arrives vacuum-packed: seven jars each hold seven characters as pairs of digits, and are ground open before baking begins. The plate is assembled back to front so that it can be served top to bottom.

Ingredients.
6053432000000 g saffron
66756579100345 g paprika
35491658878180 g turmeric
78856790565342 g cumin
31619284707488 g cardamom
27417601739411 g cloves
140764621226 g nutmeg
7 jars
7 g pod size
100 g salt
32 g pepper
8192 g flour
32769 g butter
15 g caster sugar
16 g brown sugar
0 g air
1 egg
2 yolks
45 minutes
304 g cinnamon
546 g vanilla
5690 g cherry pits
9826 g white cherries
79 wafers
37 layers
10 ml water

Cooking time: 45 minutes.

Pre-heat oven to 180 degrees Celsius.

Method.
Put nutmeg into the 4th mixing bowl. Put cloves into the 4th mixing bowl. Put cardamom into the 4th mixing bowl. Put cumin into the 4th mixing bowl. Put turmeric into the 4th mixing bowl. Put paprika into the 4th mixing bowl. Put saffron into the 4th mixing bowl.
Grate the jars. Fold spice into the 4th mixing bowl. Put pod size into the mixing bowl. Fold pods into the mixing bowl.
Crush the pods. Put spice into the mixing bowl. Divide salt into the mixing bowl. Combine salt into the mixing bowl. Fold rest into the mixing bowl. Put spice into the mixing bowl. Remove rest from the mixing bowl. Add pepper to the mixing bowl. Fold glaze into the mixing bowl. Liquefy glaze. Put glaze into the 2nd mixing bowl. Put spice into the mixing bowl. Divide salt into the mixing bowl. Fold spice into the mixing bowl. Crush the pods until crushed.
Grate the jars until grated.
Layer the layers. Put water into the 3rd mixing bowl. Put cherry pits into the mixing bowl. Fold red cherries into the mixing bowl. Put wafers into the mixing bowl. Fold wafer into the mixing bowl.
Whisk the wafer. Put air into the mixing bowl. Fold red jam into the mixing bowl. Put air into the mixing bowl. Fold white jam into the mixing bowl. Put air into the mixing bowl. Fold sprinkles into the mixing bowl. Put minutes into the mixing bowl. Fold patience into the mixing bowl.
Knead the patience. Put red jam into the mixing bowl. Combine red jam into the mixing bowl. Divide flour into the mixing bowl. Fold red syrup into the mixing bowl. Put white jam into the mixing bowl. Combine white jam into the mixing bowl. Divide flour into the mixing bowl. Fold white syrup into the mixing bowl. Put red syrup into the mixing bowl. Add white syrup to the mixing bowl. Divide butter into the mixing bowl. Add caster sugar to the mixing bowl. Divide brown sugar into the mixing bowl. Fold smoke into the mixing bowl. Put egg into the mixing bowl. Remove smoke from the mixing bowl. Fold steam into the mixing bowl. Put sprinkles into the mixing bowl. Add steam to the mixing bowl. Fold sprinkles into the mixing bowl. Put patience into the mixing bowl. Combine steam into the mixing bowl. Add smoke to the mixing bowl. Fold patience into the mixing bowl. Put red jam into the mixing bowl. Combine white jam into the mixing bowl. Combine yolks into the mixing bowl. Divide flour into the mixing bowl. Add white cherries to the mixing bowl. Fold white jam into the mixing bowl. Put red syrup into the mixing bowl. Remove white syrup from the mixing bowl. Add red cherries to the mixing bowl. Fold red jam into the mixing bowl. Knead the patience until kneaded.
Put sprinkles into the mixing bowl. Fold turns into the mixing bowl. Turn the turns. Stir sprinkles into the 2nd mixing bowl. Turn the turns until turned. Fold glaze into the 2nd mixing bowl. Put glaze into the 2nd mixing bowl. Put glaze into the 3rd mixing bowl. Stir sprinkles into the 2nd mixing bowl.
Put red cherries into the mixing bowl. Remove cinnamon from the mixing bowl. Fold red cherries into the mixing bowl. Whisk the wafer until whisked.
Put white cherries into the mixing bowl. Remove vanilla from the mixing bowl. Fold white cherries into the mixing bowl. Layer the layers until layered.
Pour contents of the 3rd mixing bowl into the baking dish.

Serves 1.
`,
  },
  "pi-pie": {
    label: "Pi Pie",
    source: `Pi Pie.

A round pie filled with 1501 decimals of pi, every one of them computed in the kitchen. The filling is made with the spigot method: a long row of apples in the second mixing bowl is swept from the far end to the near end, each apple passing a carry of dough to its neighbour, and after every sweep four fresh digits of pi drip out of the end of the row. The row of apples is flipped back and fourteen apples are eaten, because they have nothing more to give. Once the filling is ready it is spooned into a pie dish shaped by the equation of a circle, and the rim is crimped by hand into a crust.

Ingredients.
10000 g flour
2000 g sugar
5264 apples
14 cloves
376 slices
1504 pips
0 g air
1 egg
2 yolks
1000 g butter
100 g cream
10 g raisins
48 g lemon zest
46 g poppy seeds
40 g pastry
32 g steam
33 rows
67 g lattice
68 g overhang
-33 g rim
-32 g depth
961 g filling line
1089 g crust line

Cooking time: 1 hour.

Pre-heat oven to 314 degrees Celsius.

Method.
Put apples into the mixing bowl. Remove egg from the mixing bowl. Fold layers into the mixing bowl. Sprinkle the layers. Put sugar into the 2nd mixing bowl. Sprinkle the layers until sprinkled. Put air into the 2nd mixing bowl.
Put air into the mixing bowl. Fold leftover into the mixing bowl.
Bake the slices. Put air into the mixing bowl. Fold dough into the mixing bowl. Put apples into the mixing bowl. Combine yolks into the mixing bowl. Fold heat into the mixing bowl. Put apples into the mixing bowl. Fold apple into the mixing bowl.
Peel the apple. Fold filling into the 2nd mixing bowl. Put dough into the mixing bowl. Combine apple into the mixing bowl. Put filling into the mixing bowl. Combine flour into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Fold dough into the mixing bowl. Put heat into the mixing bowl. Remove egg from the mixing bowl. Fold heat into the mixing bowl. Put dough into the mixing bowl. Divide heat into the mixing bowl. Fold risen dough into the mixing bowl. Put dough into the 3rd mixing bowl. Put risen dough into the mixing bowl. Combine heat into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the 3rd mixing bowl. Put risen dough into the mixing bowl. Fold dough into the mixing bowl. Put heat into the mixing bowl. Remove egg from the mixing bowl. Fold heat into the mixing bowl. Peel the apple until peeled.
Put dough into the mixing bowl. Divide flour into the mixing bowl. Add leftover to the mixing bowl. Fold wedge into the mixing bowl. Put dough into the mixing bowl. Divide flour into the mixing bowl. Combine flour into the mixing bowl. Fold crumbs into the mixing bowl. Put dough into the mixing bowl. Remove crumbs from the mixing bowl. Fold leftover into the mixing bowl.
Put wedge into the 4th mixing bowl. Divide butter into the 4th mixing bowl. Add lemon zest to the 4th mixing bowl.
Put wedge into the 4th mixing bowl. Divide cream into the 4th mixing bowl. Put wedge into the mixing bowl. Divide butter into the mixing bowl. Combine raisins into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the 4th mixing bowl. Add lemon zest to the 4th mixing bowl.
Put wedge into the 4th mixing bowl. Divide raisins into the 4th mixing bowl. Put wedge into the mixing bowl. Divide cream into the mixing bowl. Combine raisins into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the 4th mixing bowl. Add lemon zest to the 4th mixing bowl.
Put wedge into the 4th mixing bowl. Put wedge into the mixing bowl. Divide raisins into the mixing bowl. Combine raisins into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the 4th mixing bowl. Add lemon zest to the 4th mixing bowl.
Put apples into the mixing bowl. Remove cloves from the mixing bowl. Fold apples into the mixing bowl. Put apples into the mixing bowl. Fold handful into the mixing bowl. Flip the handful. Fold filling into the 3rd mixing bowl. Put filling into the 2nd mixing bowl. Flip the handful until flipped. Clean the 3rd mixing bowl.
Bake the slices until baked.
Stack the pips. Fold piece into the 4th mixing bowl. Put piece into the 5th mixing bowl. Stack the pips until stacked. Fold piece into the 5th mixing bowl. Put poppy seeds into the 5th mixing bowl. Put piece into the 5th mixing bowl.
Put air into the mixing bowl. Fold portions into the mixing bowl. Put air into the mixing bowl. Fold parity into the mixing bowl.
Roll the rows. Put rim into the mixing bowl. Fold across into the mixing bowl. Put lattice into the mixing bowl. Fold strips into the mixing bowl. Put depth into the mixing bowl. Combine depth into the mixing bowl. Fold depth squared into the mixing bowl.
Crimp the strips. Put across into the mixing bowl. Combine across into the mixing bowl. Add depth squared to the mixing bowl. Fold distance into the mixing bowl. Put distance into the mixing bowl. Divide filling line into the mixing bowl. Add egg to the mixing bowl. Divide yolks into the mixing bowl. Fold filled into the mixing bowl. Put distance into the mixing bowl. Divide crust line into the mixing bowl. Fold crusted into the mixing bowl. Put egg into the mixing bowl. Remove filled from the mixing bowl. Fold digit flag into the mixing bowl. Put filled into the mixing bowl. Remove crusted from the mixing bowl. Fold crust flag into the mixing bowl. Put lattice into the mixing bowl. Remove across from the mixing bowl. Divide overhang into the mixing bowl. Combine crusted into the mixing bowl. Fold space flag into the mixing bowl.
Scoop the digit flag. Fold piece into the 5th mixing bowl. Put piece into the 6th mixing bowl. Set aside. Scoop until scooped.
Pinch the crust flag. Put pastry into the 6th mixing bowl. Add parity to the 6th mixing bowl. Set aside. Pinch until pinched.
Dust the space flag. Put steam into the 6th mixing bowl. Set aside. Dust until dusted.
Put portions into the mixing bowl. Add digit flag to the mixing bowl. Add crust flag to the mixing bowl. Add space flag to the mixing bowl. Fold portions into the mixing bowl. Put egg into the mixing bowl. Remove parity from the mixing bowl. Fold parity into the mixing bowl. Put across into the mixing bowl. Add egg to the mixing bowl. Fold across into the mixing bowl. Crimp the strips until crimped.
Put raisins into the 6th mixing bowl. Put portions into the mixing bowl. Add egg to the mixing bowl. Fold portions into the mixing bowl. Put depth into the mixing bowl. Add yolks to the mixing bowl. Fold depth into the mixing bowl. Roll the rows until rolled.
Plate the portions. Fold piece into the 6th mixing bowl. Put piece into the 7th mixing bowl. Plate the portions until plated.
Liquefy contents of the 7th mixing bowl. Pour contents of the 7th mixing bowl into the baking dish.

Serves 1.
`,
  },
  "mirror-glaze-bombe": {
    label: "Mirror Glaze Bombe",
    source: `Mirror Glaze Bombe on a Gingham Tablecloth.

A ray tracer disguised as a dessert. A perfectly round bombe hovers over a gingham tablecloth that stretches all the way to the horizon, glazed so shiny that the tablecloth can be seen in it. For every crumb on the plate the chef follows a ray of light from the eye into the scene. If the ray strikes the bombe, the chef finds the exact spot with a square root (refined twenty-four times by Newton's method), measures how squarely the sunlight lands there, adds a sparkle where the sun is mirrored, and bounces the ray off the glaze. Whatever the ray reaches next, the tablecloth or the sky, is added to the flavour. On the tablecloth the chef works out which gingham square was hit, and whether the bombe stands between that square and the sun; if it does, the square lies in shadow. Faraway squares fade into the haze of the horizon. The flavour is then matched to a row of fourteen liquids, from clear steam to dark chocolate. There is no "if" in a kitchen, so every decision is a loop that runs at most once; and there is no "less than" either, so every comparison is made by dividing by a pinch of sea salt the size of an ocean. Everything is measured in fixed-point grams, 1024 to the unit, with no fractions in sight.

Ingredients.
1024 g flour
1048576 g icing sugar
2305843009213693952 g sea salt
0 g air
1 cherry
2 eggs
4 quarters
9 g gloss
10 g salt
14 spices
1025 g sieve
4096 g first guess
24 rounds of patience
650 g lentils
78 g breadcrumbs
215 g tilt
79 g width
36 rows
-35 g frost
250 g eye height
-3600 g eye distance
100 g eye lift
-3686400 g dough
11642 g crust
150 g bombe height
-1024 g table height
-561 g west sun
729 g high sun
-280 g south sun
-855846 g noon shadow
329700 g tablecloth
65536 g gingham offset
700 g gingham
110 g dye
890 g bleach
8192 g haze
260 g sky blue
780 g sky fade
409 g mirror glaze
10 ml water
32 ml steam
46 ml milk
44 ml cream
58 ml syrup
59 ml honey
45 ml juice
61 ml broth
43 ml wine
42 ml rum
111 ml brandy
35 ml coffee
37 ml cocoa
38 ml ganache
64 ml chocolate

Cooking time: 3 hours.

Pre-heat oven to 220 degrees Celsius (gas mark 7).

Method.
Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put chocolate into the 2nd mixing bowl. Put ganache into the 2nd mixing bowl. Put cocoa into the 2nd mixing bowl. Put coffee into the 2nd mixing bowl. Put brandy into the 2nd mixing bowl. Put rum into the 2nd mixing bowl. Put wine into the 2nd mixing bowl. Put broth into the 2nd mixing bowl. Put juice into the 2nd mixing bowl. Put honey into the 2nd mixing bowl. Put syrup into the 2nd mixing bowl. Put cream into the 2nd mixing bowl. Put milk into the 2nd mixing bowl. Put steam into the 2nd mixing bowl.
Put frost into the mixing bowl. Fold row lift into the mixing bowl.
Chill the rows. Put water into the 3rd mixing bowl. Put row lift into the mixing bowl. Combine lentils into the mixing bowl. Divide breadcrumbs into the mixing bowl. Combine eggs into the mixing bowl. Remove tilt from the mixing bowl. Fold row ray into the mixing bowl. Put breadcrumbs into the mixing bowl. Fold column lift into the mixing bowl. Put width into the mixing bowl. Fold columns into the mixing bowl.
Pipe the columns. Put column lift into the mixing bowl. Combine lentils into the mixing bowl. Divide breadcrumbs into the mixing bowl. Fold ray x into the mixing bowl. Put row ray into the mixing bowl. Fold ray y into the mixing bowl. Put flour into the mixing bowl. Fold ray z into the mixing bowl. Put air into the mixing bowl. Fold eye x into the mixing bowl. Put eye height into the mixing bowl. Fold eye y into the mixing bowl. Put eye distance into the mixing bowl. Fold eye z into the mixing bowl. Put air into the mixing bowl. Fold brightness into the mixing bowl. Put flour into the mixing bowl. Fold sheen into the mixing bowl.
Put ray x into the mixing bowl. Combine ray x into the mixing bowl. Put ray y into the mixing bowl. Combine ray y into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Add icing sugar to the mixing bowl. Divide flour into the mixing bowl. Fold span into the mixing bowl. Put ray y into the mixing bowl. Combine eye lift into the mixing bowl. Add dough to the mixing bowl. Divide flour into the mixing bowl. Fold bias into the mixing bowl. Put bias into the mixing bowl. Combine bias into the mixing bowl. Put span into the mixing bowl. Combine crust into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the mixing bowl. Fold batter into the mixing bowl. Put batter into the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Fold contact into the mixing bowl.
Dip the contact. Put first guess into the mixing bowl. Fold root into the mixing bowl. Put rounds of patience into the mixing bowl. Fold rounds into the mixing bowl. Refine the rounds. Put batter into the mixing bowl. Divide root into the mixing bowl. Add root to the mixing bowl. Add cherry to the mixing bowl. Divide eggs into the mixing bowl. Fold root into the mixing bowl. Refine the rounds until refined.
Put air into the mixing bowl. Remove bias from the mixing bowl. Remove root from the mixing bowl. Combine flour into the mixing bowl. Divide span into the mixing bowl. Fold depth into the mixing bowl. Put depth into the mixing bowl. Combine ray x into the mixing bowl. Divide flour into the mixing bowl. Fold eye x into the mixing bowl. Put depth into the mixing bowl. Combine ray y into the mixing bowl. Divide flour into the mixing bowl. Add eye y to the mixing bowl. Fold eye y into the mixing bowl. Put depth into the mixing bowl. Add eye z to the mixing bowl. Fold eye z into the mixing bowl. Put eye y into the mixing bowl. Remove bombe height from the mixing bowl. Fold normal y into the mixing bowl.
Put eye x into the mixing bowl. Combine west sun into the mixing bowl. Put normal y into the mixing bowl. Combine high sun into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Put eye z into the mixing bowl. Combine south sun into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Divide flour into the mixing bowl. Fold light into the mixing bowl. Put light into the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Combine light into the mixing bowl. Fold light into the mixing bowl.
Put ray x into the mixing bowl. Combine eye x into the mixing bowl. Put ray y into the mixing bowl. Combine normal y into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Put flour into the mixing bowl. Combine eye z into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Divide flour into the mixing bowl. Fold bounce into the mixing bowl.
Put bounce into the mixing bowl. Combine eggs into the mixing bowl. Combine eye x into the mixing bowl. Divide flour into the mixing bowl. Fold crumbs into the mixing bowl. Put ray x into the mixing bowl. Remove crumbs from the mixing bowl. Fold ray x into the mixing bowl. Put bounce into the mixing bowl. Combine eggs into the mixing bowl. Combine normal y into the mixing bowl. Divide flour into the mixing bowl. Fold crumbs into the mixing bowl. Put ray y into the mixing bowl. Remove crumbs from the mixing bowl. Fold ray y into the mixing bowl. Put bounce into the mixing bowl. Combine eggs into the mixing bowl. Combine eye z into the mixing bowl. Divide flour into the mixing bowl. Fold crumbs into the mixing bowl. Put flour into the mixing bowl. Remove crumbs from the mixing bowl. Fold ray z into the mixing bowl.
Put ray x into the mixing bowl. Combine west sun into the mixing bowl. Put ray y into the mixing bowl. Combine high sun into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Put ray z into the mixing bowl. Combine south sun into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Divide flour into the mixing bowl. Fold shine into the mixing bowl. Put ray x into the mixing bowl. Combine ray x into the mixing bowl. Put ray y into the mixing bowl. Combine ray y into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Put ray z into the mixing bowl. Combine ray z into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Fold length into the mixing bowl.
Put shine into the mixing bowl. Combine shine into the mixing bowl. Combine flour into the mixing bowl. Divide length into the mixing bowl. Fold sparkle into the mixing bowl. Put shine into the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Combine sparkle into the mixing bowl. Fold sparkle into the mixing bowl. Put sparkle into the mixing bowl. Combine sparkle into the mixing bowl. Divide flour into the mixing bowl. Fold sparkle into the mixing bowl. Put sparkle into the mixing bowl. Combine sparkle into the mixing bowl. Divide flour into the mixing bowl. Fold sparkle into the mixing bowl. Put sparkle into the mixing bowl. Combine sparkle into the mixing bowl. Divide flour into the mixing bowl. Fold sparkle into the mixing bowl.
Put light into the mixing bowl. Combine gloss into the mixing bowl. Divide salt into the mixing bowl. Add sparkle to the mixing bowl. Fold brightness into the mixing bowl. Put mirror glaze into the mixing bowl. Fold sheen into the mixing bowl. Set aside. Dip until dipped.
Put air into the mixing bowl. Remove ray y from the mixing bowl. Remove cherry from the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Fold table flag into the mixing bowl. Put cherry into the mixing bowl. Remove table flag from the mixing bowl. Fold sky flag into the mixing bowl.
Spread the table flag. Put table height into the mixing bowl. Remove eye y from the mixing bowl. Combine flour into the mixing bowl. Divide ray y into the mixing bowl. Fold depth into the mixing bowl. Put depth into the mixing bowl. Combine ray x into the mixing bowl. Divide flour into the mixing bowl. Add eye x to the mixing bowl. Fold spot x into the mixing bowl. Put depth into the mixing bowl. Combine ray z into the mixing bowl. Divide flour into the mixing bowl. Add eye z to the mixing bowl. Fold spot z into the mixing bowl.
Put spot x into the mixing bowl. Add gingham offset to the mixing bowl. Divide gingham into the mixing bowl. Put spot z into the mixing bowl. Add gingham offset to the mixing bowl. Divide gingham into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Fold check into the mixing bowl. Put check into the mixing bowl. Put check into the mixing bowl. Divide eggs into the mixing bowl. Combine eggs into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the mixing bowl. Fold check into the mixing bowl. Put check into the mixing bowl. Combine check into the mixing bowl. Combine bleach into the mixing bowl. Add dye to the mixing bowl. Fold base into the mixing bowl.
Put spot x into the mixing bowl. Combine west sun into the mixing bowl. Put spot z into the mixing bowl. Combine south sun into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Add noon shadow to the mixing bowl. Divide flour into the mixing bowl. Fold shade into the mixing bowl. Put spot x into the mixing bowl. Combine spot x into the mixing bowl. Put spot z into the mixing bowl. Combine spot z into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Add tablecloth to the mixing bowl. Divide flour into the mixing bowl. Fold cloth into the mixing bowl. Put shade into the mixing bowl. Combine shade into the mixing bowl. Put cloth into the mixing bowl. Combine flour into the mixing bowl. Fold crumbs into the mixing bowl. Remove crumbs from the mixing bowl. Fold eclipse into the mixing bowl.
Put eclipse into the mixing bowl. Remove cherry from the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Fold umbra into the mixing bowl. Put air into the mixing bowl. Remove shade from the mixing bowl. Remove cherry from the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Combine umbra into the mixing bowl. Fold umbra into the mixing bowl. Put base into the mixing bowl. Divide quarters into the mixing bowl. Fold crumbs into the mixing bowl. Put base into the mixing bowl. Remove crumbs from the mixing bowl. Combine umbra into the mixing bowl. Fold crumbs into the mixing bowl. Put base into the mixing bowl. Remove crumbs from the mixing bowl. Fold base into the mixing bowl.
Put haze into the mixing bowl. Add depth to the mixing bowl. Fold fog into the mixing bowl. Put base into the mixing bowl. Combine haze into the mixing bowl. Put depth into the mixing bowl. Combine sky blue into the mixing bowl. Fold crumbs into the mixing bowl. Add crumbs to the mixing bowl. Divide fog into the mixing bowl. Fold view into the mixing bowl. Set aside. Spread until spread.
Paint the sky flag. Put ray y into the mixing bowl. Combine sky fade into the mixing bowl. Divide flour into the mixing bowl. Fold crumbs into the mixing bowl. Put sky blue into the mixing bowl. Remove crumbs from the mixing bowl. Fold view into the mixing bowl. Put view into the mixing bowl. Add sea salt to the mixing bowl. Divide sea salt into the mixing bowl. Combine view into the mixing bowl. Fold view into the mixing bowl. Set aside. Paint until painted.
Put view into the mixing bowl. Combine sheen into the mixing bowl. Divide flour into the mixing bowl. Add brightness to the mixing bowl. Fold brightness into the mixing bowl. Put brightness into the mixing bowl. Combine spices into the mixing bowl. Divide sieve into the mixing bowl. Fold pick into the mixing bowl.
Put pick into the mixing bowl. Fold turns into the mixing bowl. Turn the turns. Stir pick into the 2nd mixing bowl. Turn the turns until turned. Fold dab into the 2nd mixing bowl. Put dab into the 2nd mixing bowl. Put dab into the 3rd mixing bowl. Stir pick into the 2nd mixing bowl.
Put column lift into the mixing bowl. Remove eggs from the mixing bowl. Fold column lift into the mixing bowl. Pipe the columns until piped.
Put row lift into the mixing bowl. Add eggs to the mixing bowl. Fold row lift into the mixing bowl. Chill the rows until chilled.
Pour contents of the 3rd mixing bowl into the baking dish.

Serves 1.
`,
  },
};

const DEFAULT_EXAMPLE = "hello-world";

const outputEl = document.getElementById("output");
const statusEl = document.getElementById("status");
const runBtn = document.getElementById("run");
const autorunEl = document.getElementById("autorun");
const examplesEl = document.getElementById("examples");
const stdinEl = document.getElementById("stdin");
const wrapEl = document.getElementById("wrap");
const shortcutEl = document.getElementById("run-shortcut");

let editor;
let ready = false;
let debounceTimer = null;

// CodeMirror theme wired entirely to the page's custom properties (see
// ../assets/site.css), so it follows whichever palette is active. A real
// EditorView.theme() is needed to beat CodeMirror's default light theme (its
// generated `.ͼ2` classes otherwise win on specificity).
const cookTheme = EditorView.theme({
  "&": { backgroundColor: "var(--surface)", color: "var(--text)", height: "100%" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    fontFamily: "var(--font-mono)",
    fontSize: "14px",
    lineHeight: "1.65",
  },
  ".cm-content": { caretColor: "var(--accent)", padding: "12px 0" },
  ".cm-line": { padding: "0 16px 0 8px" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--accent)", borderLeftWidth: "2px" },
  ".cm-gutters": {
    backgroundColor: "var(--surface)",
    color: "var(--muted)",
    border: "none",
  },
  ".cm-lineNumbers .cm-gutterElement": {
    color: "var(--muted)",
    opacity: "0.7",
    padding: "0 4px 0 16px",
    minWidth: "40px",
  },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--text) 4%, transparent)" },
  ".cm-activeLineGutter": { backgroundColor: "transparent" },
  ".cm-activeLineGutter.cm-gutterElement": { color: "var(--text)", opacity: "1" },
  ".cm-foldGutter .cm-gutterElement": { color: "var(--muted)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
    { backgroundColor: "color-mix(in srgb, var(--accent) 22%, transparent)" },
  ".cm-selectionMatch": { backgroundColor: "color-mix(in srgb, var(--accent) 12%, transparent)" },
  "&.cm-focused .cm-matchingBracket": {
    backgroundColor: "color-mix(in srgb, var(--accent) 18%, transparent)",
    outline: "none",
  },
});

// Chef syntax highlighting: the shared tokenizer (also used by the cookbook)
// returns sorted, non-overlapping ranges, which become `tok-*` class marks.
// Recipes are small, so the whole document is re-tokenized on each change.
const tokenMarks = {};

function chefDecorations(doc) {
  const builder = new RangeSetBuilder();
  for (const { from, to, type } of tokenizeChef(doc.toString())) {
    tokenMarks[type] ??= Decoration.mark({ class: "tok-" + type });
    builder.add(from, to, tokenMarks[type]);
  }
  return builder.finish();
}

const chefHighlighting = ViewPlugin.fromClass(
  class {
    constructor(view) {
      this.decorations = chefDecorations(view.state.doc);
    }
    update(update) {
      if (update.docChanged) this.decorations = chefDecorations(update.state.doc);
    }
  },
  { decorations: (plugin) => plugin.decorations },
);

const WRAP_STORAGE_KEY = "cheffers-wrap";

// Line wrapping is toggled live by reconfiguring this compartment. Chef
// methods are traditionally written as one long line, so wrapping is on by
// default; the choice persists like the theme.
const wrapCompartment = new Compartment();

function storedWrap() {
  try {
    return localStorage.getItem(WRAP_STORAGE_KEY) !== "0";
  } catch {
    return true;
  }
}

function wrapExtension(enabled) {
  return enabled ? EditorView.lineWrapping : [];
}

function applyWrap(enabled) {
  editor.dispatch({
    effects: wrapCompartment.reconfigure(wrapExtension(enabled)),
  });
  try {
    localStorage.setItem(WRAP_STORAGE_KEY, enabled ? "1" : "0");
  } catch {
    /* storage may be unavailable; the setting still applies this session */
  }
}

// A recipe passed in the URL fragment (the cookbook's "Open in playground"
// links): #recipe=<base64url of JSON {c: source, i: input}>. Mirrors
// encodeRecipeHash in docs/cookbook/cookbook.js — keep the two in sync.
function decodeRecipeHash() {
  const match = /[#&]recipe=([A-Za-z0-9_-]+)/.exec(location.hash);
  if (!match) return null;
  try {
    const b64 = match[1].replace(/-/g, "+").replace(/_/g, "/");
    const bytes = Uint8Array.from(atob(b64), (ch) => ch.charCodeAt(0));
    const parsed = JSON.parse(new TextDecoder().decode(bytes));
    if (typeof parsed.c !== "string") return null;
    return { source: parsed.c, input: typeof parsed.i === "string" ? parsed.i : "" };
  } catch {
    return null;
  }
}

function setStatus(text, kind) {
  statusEl.textContent = text;
  statusEl.className = "status" + (kind ? " " + kind : "");
}

function render(result) {
  if (result == null) {
    outputEl.textContent = "Internal error: no result returned.";
    outputEl.classList.add("error");
    setStatus("error", "err");
    return;
  }

  if (result.ok) {
    outputEl.textContent = result.output.length ? result.output : "(no output)";
    outputEl.classList.remove("error");
    setStatus("ok", "ok");
  } else {
    // Show any partial output, then the rich (ANSI-colored) error beneath it.
    let html = "";
    if (result.output.length) html += escapeHtml(result.output) + "\n\n";
    html += ansiToHtml(result.error);
    outputEl.innerHTML = html;
    outputEl.classList.add("error");
    setStatus("error", "err");
  }
}

function runNow() {
  if (!ready) return;
  const source = editor.state.doc.toString();
  try {
    // The input box stands in for stdin: whitespace-separated numbers, one
    // consumed per "Take ... from refrigerator" instruction.
    render(run_chef(source, stdinEl.value));
  } catch (err) {
    outputEl.textContent = "Failed to run interpreter: " + err;
    outputEl.classList.add("error");
    setStatus("error", "err");
  }
}

function scheduleRun() {
  if (!autorunEl.checked) return;
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(runNow, 400);
}

// ⌘↵ / Ctrl↵ runs the recipe from anywhere on the page. Inside the editor it
// must outrank basicSetup's own Mod-Enter (insert blank line).
const IS_MAC = /mac|iphone|ipad/i.test(navigator.userAgentData?.platform ?? navigator.platform);

function runShortcut() {
  runNow();
  return true;
}

function onPageKeydown(event) {
  // The editor's keymap already handled (and prevented) its own presses.
  if (event.defaultPrevented) return;
  if (event.key === "Enter" && (IS_MAC ? event.metaKey : event.ctrlKey)) {
    event.preventDefault();
    runNow();
  }
}

function buildEditor(initialDoc) {
  editor = new EditorView({
    doc: initialDoc,
    extensions: [
      Prec.highest(keymap.of([{ key: "Mod-Enter", run: runShortcut }])),
      basicSetup,
      cookTheme,
      chefHighlighting,
      wrapCompartment.of(wrapExtension(wrapEl.checked)),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) scheduleRun();
      }),
    ],
    parent: document.getElementById("editor"),
  });
}

function setEditorContent(text) {
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: text },
  });
}

function populateExamples() {
  for (const [key, { label }] of Object.entries(EXAMPLES)) {
    const opt = document.createElement("option");
    opt.value = key;
    opt.textContent = label;
    examplesEl.appendChild(opt);
  }
  examplesEl.value = DEFAULT_EXAMPLE;
}

async function main() {
  shortcutEl.textContent = IS_MAC ? "⌘↵" : "Ctrl ↵";
  runBtn.setAttribute("aria-keyshortcuts", IS_MAC ? "Meta+Enter" : "Control+Enter");
  runBtn.title = `Run the recipe (${IS_MAC ? "⌘ Return" : "Ctrl+Enter"})`;

  populateExamples();
  wrapEl.checked = storedWrap();

  // A recipe in the URL fragment (from the cookbook) beats the default
  // example; it also gets its own entry in the examples dropdown so the
  // selection reflects what's loaded.
  const shared = decodeRecipeHash();
  if (shared) {
    const opt = document.createElement("option");
    opt.value = "__shared";
    opt.textContent = "From the cookbook";
    examplesEl.appendChild(opt);
    examplesEl.value = "__shared";
    stdinEl.value = shared.input;
  }
  buildEditor(shared ? shared.source : EXAMPLES[DEFAULT_EXAMPLE].source);
  wrapEl.addEventListener("change", () => applyWrap(wrapEl.checked));

  setStatus("loading interpreter…", "busy");
  await init();
  ready = true;
  setStatus("");

  runBtn.addEventListener("click", runNow);
  document.addEventListener("keydown", onPageKeydown);
  stdinEl.addEventListener("input", scheduleRun);
  examplesEl.addEventListener("change", () => {
    const example = EXAMPLES[examplesEl.value];
    if (example) {
      setEditorContent(example.source);
      stdinEl.value = example.input ?? "";
      runNow();
    }
  });

  // Run once on load so the user immediately sees output.
  runNow();
}

main();
