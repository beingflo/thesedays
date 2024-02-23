import type { Component } from 'solid-js';

const Login: Component = () => {
  return (
    <div class="mx-auto flex flex-col w-1/2 pt-12">
      <div class="flex flex-row gap-4 items-baseline">
        <p class="text-4xl md:text-6xl mb-4 text-black dark:text-white font-extrabold">
          Login
        </p>
        <a
          href="/signup"
          class="text-md md:text-lg text-gray-800 dark:text-white w-fit h-fit"
        >
          Signup
        </a>
      </div>
    </div>
  );
};

export default Login;
