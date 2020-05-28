# Getting Started

It is recommended to split the application and it's deployment into different crates.
This has the advantage that the same application can be easily deployed with several different methods but the 
structure of the repository can be kept clean.
However to simplify the setup of this tutorial, we will skip this step.

Futhermore, we will use [electron](https://www.electronjs.org/) to deploy a desktop application.
Currently, this is the recommended method to deploy `greenhorn` applications.
To simplify the project setup, start by cloning the quickstart repository:

```bash
git clone https://github.com/raffber/greenhorn-electron-quickstart
```

This quickstart repo makes us of the following tools:
 * [neon](https://neon-bindings.com/) to compile rust modules for nodejs
 * [webpack](https://webpack.js.org/) to bundle JS and CSS assets
 * [electron](https://www.electronjs.org/) as browser frontend
 * [electron-builder](https://github.com/electron-userland/electron-builder) to package the app into an installer

Then, setup the nodejs environment (make sure you have `nodejs` and `npm` installed):

```bash
npm install
```

Compiling and running the application can be achieved using `npm`:

```bash
npm run start
```

This will compile the application in debug mode and run electron.
Finally, if you want to package the application, use:

```bash
npm run package
```

That's it for the project setup. You are now ready to start developping you app.
