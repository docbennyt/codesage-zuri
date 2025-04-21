"""White Byte (Level 1) - Fundamental Python Concepts"""

WHITE_BYTE = {
    'title': '⚪ White Byte',
    'description': 'Master the fundamentals of Python programming',
    'topics': {
        'fundamentals': {
            'title': 'Basic Building Blocks',
            'concepts': {
                'variables': {
                    'title': 'Variables and Data Types',
                    'content': '''Variables are containers for storing data values. In Python, you don't need to declare the type of variable.
                    
Basic Data Types:
- Integer (int): Whole numbers (e.g., 42, -7)
- Float (float): Decimal numbers (e.g., 3.14, -0.001)
- String (str): Text (e.g., "Hello", 'Python')
- Boolean (bool): True or False
- None: Represents absence of value''',
                    'examples': [
                        '''# Variable assignment
age = 25
pi = 3.14159
name = "Alice"
is_student = True
no_value = None''',
                        '''# Type checking
print(type(age))    # <class 'int'>
print(type(pi))     # <class 'float'>
print(type(name))   # <class 'str'>
print(type(is_student))  # <class 'bool'>'''
                    ]
                },
                'operators': {
                    'title': 'Basic Operators',
                    'content': '''Python supports various types of operators:
                    
Arithmetic Operators:
- + (Addition)
- - (Subtraction)
- * (Multiplication)
- / (Division)
- % (Modulus)
- ** (Exponentiation)
- // (Floor Division)

Comparison Operators:
- == (Equal)
- != (Not Equal)
- > (Greater Than)
- < (Less Than)
- >= (Greater Than or Equal)
- <= (Less Than or Equal)

Logical Operators:
- and
- or
- not''',
                    'examples': [
                        '''# Arithmetic operations
x = 10
y = 3
print(x + y)   # 13
print(x - y)   # 7
print(x * y)   # 30
print(x / y)   # 3.333...
print(x % y)   # 1
print(x ** y)  # 1000
print(x // y)  # 3''',
                        '''# Comparison and logical operators
a = 5
b = 10
print(a == b)      # False
print(a != b)      # True
print(a > b)       # False
print(a < b)       # True
print(a >= b)      # False
print(a <= b)      # True
print(a > 0 and b > 0)  # True
print(a > 0 or b < 0)   # True
print(not a > b)        # True'''
                    ]
                }
            }
        },
        'control_flow': {
            'title': 'Control Flow',
            'concepts': {
                'if_else': {
                    'title': 'If-Else Statements',
                    'content': '''If-else statements allow you to execute different blocks of code based on conditions.

Syntax:
if condition:
    # code block
elif condition:
    # code block
else:
    # code block''',
                    'examples': [
                        '''# Basic if-else
age = 18
if age >= 18:
    print("You are an adult")
else:
    print("You are a minor")''',
                        '''# Multiple conditions
score = 85
if score >= 90:
    print("Grade: A")
elif score >= 80:
    print("Grade: B")
elif score >= 70:
    print("Grade: C")
else:
    print("Grade: F")'''
                    ]
                }
            }
        }
    }
}
