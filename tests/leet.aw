// // LeetCode-style problems in Auwla
// // Testing medium JavaScript logic

// // Problem 1: Two Sum
// // Given an array of integers and a target sum, return indices of the two numbers that add up to target
// fn two_sum(nums: number[], target: number): number[] {
//     // Create a map to store numbers and their indices
//     let num_map = {};
    
//     for i in 0..nums.len() {
//         let complement = target - nums[i];
        
//         // Check if complement exists in map
//         if complement in num_map {
//             return [num_map[complement], i];
//         }
        
//         // Store current number and its index
//         num_map[nums[i]] = i;
//     }
    
//     // Return empty array if no solution found
//     return [];
// }

// // Test Two Sum
// let nums1 = [2, 7, 11, 15];
// let target1 = 9;
// let result1 = two_sum(nums1, target1);
// print("Two Sum Test 1: {result1}"); // Should print [0, 1]

// let nums2 = [3, 2, 4];
// let target2 = 6;
// let result2 = two_sum(nums2, target2);
// print("Two Sum Test 2: {result2}"); // Should print [1, 2]

// let nums3 = [3, 3];
// let target3 = 6;
// let result3 = two_sum(nums3, target3);
// print("Two Sum Test 3: {result3}"); // Should print [0, 1]

// // Problem 2: Valid Parentheses
// // Given a string containing just parentheses, determine if the input string is valid
// fn is_valid_parentheses(s: string): bool {
//     let stack = [];
//     let pairs = {
//         ')': '(',
//         ']': '[',
//         '}': '{'
//     };
    
//     for ch in s {
//         if ch in pairs {
//             // Closing bracket
//             if stack.len() == 0 || stack[stack.len() - 1] != pairs[char] {
//                 return false;
//             }
//             stack.pop();
//         } else {
//             // Opening bracket
//             stack.push(ch);
//         }
//     }
    
//     return stack.len() == 0;
// }

// // Test Valid Parentheses
// print("Valid Parentheses Tests:");
// print("() : {is_valid_parentheses("()")}"); // Should print true
// print("()[]{} : {is_valid_parentheses("()[]{}")}"); // Should print true
// print("(] : {is_valid_parentheses("(]")}"); // Should print false
// print("([)] : {is_valid_parentheses("([)]")}"); // Should print false
// print("{[]} : {is_valid_parentheses("{[]}")}"); // Should print true

// // Problem 3: Array Chunking / Sliding Window - Max Sum Subarray of Size K
// // Given an array of integers and a positive integer k, find the maximum sum of any contiguous subarray of size k
// fn max_subarray_sum(arr: number[], k: number): number {
//     if arr.len() < k {
//         return 0;
//     }
    
//     // Calculate initial window sum
//     var max_sum = 0;
//     var window_sum = 0;
    
//     for i in 0..k {
//         window_sum = window_sum + arr[i];
//     }
//     max_sum = window_sum;
    
//     // Slide the window
//     for i in k..arr.len() {
//         window_sum = window_sum - arr[i - k] + arr[i];
//         if window_sum > max_sum {
//             max_sum = window_sum;
//         }
//     }
    
//     return max_sum;
// }

// // Test Max Subarray Sum
// print("\nMax Subarray Sum Tests:");
// let arr1 = [2, 1, 5, 1, 3, 2];
// let k1 = 3;
// print("Array: {arr1}, k={k1}, Max Sum: {max_subarray_sum(arr1, k1)}"); // Should print 9 (subarray [5, 1, 3])

// let arr2 = [2, 3, 4, 1, 5];
// let k2 = 2;
// print("Array: {arr2}, k={k2}, Max Sum: {max_subarray_sum(arr2, k2)}"); // Should print 7 (subarray [3, 4])

// // Problem 4: Reverse a String
// fn reverse_string(s: string): string {
//     let chars = s.split("");
//     let left = 0;
//     let right = chars.len() - 1;
    
//     while left < right {
//         let temp = chars[left];
//         chars[left] = chars[right];
//         chars[right] = temp;
//         left = left + 1;
//         right = right - 1;
//     }
    
//     return chars.join("");
// }

// // Test Reverse String
// print("\nReverse String Tests:");
// print("hello -> {reverse_string("hello")}"); // Should print olleh
// print("Auwla -> {reverse_string("Auwla")}"); // Should print alwuA
// print("racecar -> {reverse_string("racecar")}"); // Should print racecar

// // Problem 5: Factorial (Recursive)
// fn factorial(n: number): number {
//     if n <= 1 {
//         return 1;
//     }
//     return n * factorial(n - 1);
// }

// // Test Factorial
// print("\nFactorial Tests:");
// print("factorial(5) = {factorial(5)}"); // Should print 120
// print("factorial(0) = {factorial(0)}"); // Should print 1
// print("factorial(7) = {factorial(7)}"); // Should print 5040


// extend string{
//     fn ones(self):string => self;
// }



