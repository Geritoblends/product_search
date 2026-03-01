# Product search API

**Correr el server**
`cargo run --release`

**Agregar un producto**
`http POST localhost:3000/products name="nombre del producto"`

**Eliminar un producto**
`http DELETE localhost:3000/products name="nombre del producto"`

**Buscar un producto similar**
`http POST localhost:3000/getSimilar query="chetos flaming hot"`

**Limpiar la base de datos de productos**
`http POST localhost:3000/products/clear`

