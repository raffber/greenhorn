console.log('Hello, World!')

class Patch {
    constructor(patch) {
        this.patch = patch;
    }

    apply(element) {
        console.log("apply")
    }
}

module.exports.Patch = Patch;