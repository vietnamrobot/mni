// Sample JavaScript for testing minification
function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

class Calculator {
    constructor() {
        this.result = 0;
        this.history = [];
    }

    add(a, b) {
        this.result = a + b;
        this.history.push({ operation: 'add', result: this.result });
        return this.result;
    }

    multiply(a, b) {
        this.result = a * b;
        this.history.push({ operation: 'multiply', result: this.result });
        return this.result;
    }

    getHistory() {
        return this.history;
    }
}

const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const doubled = numbers.map(n => n * 2);
const filtered = numbers.filter(n => n % 2 === 0);
const sum = numbers.reduce((acc, n) => acc + n, 0);

console.log('Doubled:', doubled);
console.log('Filtered:', filtered);
console.log('Sum:', sum);

const calc = new Calculator();
calc.add(10, 20);
calc.multiply(5, 6);
console.log('Calculator history:', calc.getHistory());

// Dead code that should be removed
if (false) {
    console.log('This should never run');
    const unused = 'This is unused';
}

// Constant expressions that can be folded
const x = 5 + 10;
const y = 20 * 2;
const z = x + y;

export { fibonacci, Calculator };
